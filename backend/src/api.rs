//! REST API endpoints

use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{Duration, Utc, TimeZone, NaiveTime};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info};

use crate::db::Database;
use crate::fhir::FhirBundle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSettings {
    pub inactivity_seconds: u64,
    pub sound_threshold: i32,
}

pub struct AppState {
    pub db: Database,
    pub base_url: String,
    pub settings: Arc<RwLock<MonitorSettings>>,
}

#[derive(Debug, Deserialize)]
pub struct ListObservationsQuery {
    #[serde(default = "default_limit")]
    pub _count: usize,
    pub minutes: Option<i64>,
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl ApiError {
    fn not_found(msg: &str) -> Self {
        Self { error: "not_found".to_string(), message: msg.to_string() }
    }
    
    fn internal_error(msg: &str) -> Self {
        Self { error: "internal_error".to_string(), message: msg.to_string() }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryResponse {
    pub total_readings: u64,
    pub fall_alerts: u64,
    pub inactivity_alerts: u64,
    pub system_status: String,
    pub last_updated: String,
}

#[get("/api/observations")]
pub async fn list_observations(
    state: web::Data<AppState>,
    query: web::Query<ListObservationsQuery>,
) -> impl Responder {
    debug!("GET /api/observations");
    
    let limit = query._count.min(1000).max(1);
    
    let result = if let Some(minutes) = query.minutes {
        let end = Utc::now();
        let start = end - Duration::minutes(minutes);
        state.db.get_readings_in_range(start, end).await
    } else {
        state.db.get_recent_readings(limit).await
    };
    
    match result {
        Ok(events) => {
            let bundle = FhirBundle::from_events(events, &state.base_url);
            HttpResponse::Ok()
                .content_type("application/fhir+json")
                .json(bundle)
        }
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::internal_error("Failed to retrieve observations"))
        }
    }
}

#[get("/api/observations/latest")]
pub async fn get_latest_observation(state: web::Data<AppState>) -> impl Responder {
    debug!("GET /api/observations/latest");
    
    match state.db.get_recent_readings(1).await {
        Ok(events) => {
            if let Some(event) = events.into_iter().next() {
                let observation = event.to_fhir(&state.base_url);
                HttpResponse::Ok()
                    .content_type("application/fhir+json")
                    .json(observation)
            } else {
                HttpResponse::NotFound()
                    .json(ApiError::not_found("No observations recorded yet"))
            }
        }
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::internal_error("Failed to retrieve observation"))
        }
    }
}

#[get("/api/observations/{id}")]
pub async fn get_observation_by_id(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> impl Responder {
    let id = path.into_inner();
    debug!("GET /api/observations/{}", id);
    
    match state.db.get_reading_by_id(id).await {
        Ok(Some(event)) => {
            let observation = event.to_fhir(&state.base_url);
            HttpResponse::Ok()
                .content_type("application/fhir+json")
                .json(observation)
        }
        Ok(None) => {
            HttpResponse::NotFound()
                .json(ApiError::not_found(&format!("Observation {} not found", id)))
        }
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::internal_error("Failed to retrieve observation"))
        }
    }
}

#[get("/api/summary")]
pub async fn get_summary(state: web::Data<AppState>) -> impl Responder {
    debug!("GET /api/summary");
    
    match state.db.get_alert_summary().await {
        Ok(summary) => {
            HttpResponse::Ok().json(SummaryResponse {
                total_readings: summary.total_readings,
                fall_alerts: summary.fall_alerts,
                inactivity_alerts: summary.inactivity_alerts,
                system_status: "active".to_string(),
                last_updated: Utc::now().to_rfc3339(),
            })
        }
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::internal_error("Failed to retrieve summary"))
        }
    }
}

#[get("/api/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

/// Query params for activity analysis
#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    /// Start hour (0-23), default 22 (10 PM)
    pub start_hour: Option<u32>,
    /// End hour (0-23), default 6 (6 AM)  
    pub end_hour: Option<u32>,
    /// Date in YYYY-MM-DD format, default today
    pub date: Option<String>,
}

/// GET /api/activity/sleep
/// 
/// Analyze sleep activity (default 10 PM to 6 AM)
/// Example: /api/activity/sleep?start_hour=22&end_hour=6&date=2024-01-15
#[get("/api/activity/sleep")]
pub async fn get_sleep_analysis(
    state: web::Data<AppState>,
    query: web::Query<ActivityQuery>,
) -> impl Responder {
    debug!("GET /api/activity/sleep");
    
    let start_hour = query.start_hour.unwrap_or(22);
    let end_hour = query.end_hour.unwrap_or(6);
    
    // Parse date or use today
    let base_date = if let Some(date_str) = &query.date {
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive())
    } else {
        Utc::now().date_naive()
    };
    
    // Calculate start and end times
    let start = Utc.from_utc_datetime(
        &base_date.and_time(NaiveTime::from_hms_opt(start_hour, 0, 0).unwrap())
    );
    
    // If end_hour < start_hour, it's the next day
    let end_date = if end_hour < start_hour {
        base_date + chrono::Duration::days(1)
    } else {
        base_date
    };
    let end = Utc.from_utc_datetime(
        &end_date.and_time(NaiveTime::from_hms_opt(end_hour, 0, 0).unwrap())
    );
    
    match state.db.get_activity_analysis(start, end).await {
        Ok(analysis) => HttpResponse::Ok().json(analysis),
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::internal_error("Failed to analyze activity"))
        }
    }
}

/// GET /api/activity/period
/// 
/// Analyze activity for custom time period
/// Example: /api/activity/period?minutes=60 (last 60 minutes)
#[get("/api/activity/period")]
pub async fn get_period_analysis(
    state: web::Data<AppState>,
    query: web::Query<ListObservationsQuery>,
) -> impl Responder {
    debug!("GET /api/activity/period");
    
    let minutes = query.minutes.unwrap_or(60);
    let end = Utc::now();
    let start = end - Duration::minutes(minutes);
    
    match state.db.get_activity_analysis(start, end).await {
        Ok(analysis) => HttpResponse::Ok().json(analysis),
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::internal_error("Failed to analyze activity"))
        }
    }
}

/// GET /api/activity/hourly
/// 
/// Get hourly activity breakdown for a day
/// Example: /api/activity/hourly?date=2024-01-15
#[get("/api/activity/hourly")]
pub async fn get_hourly_analysis(
    state: web::Data<AppState>,
    query: web::Query<ActivityQuery>,
) -> impl Responder {
    debug!("GET /api/activity/hourly");
    
    let date = if let Some(date_str) = &query.date {
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()))
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };
    
    match state.db.get_hourly_activity(date).await {
        Ok(hourly) => HttpResponse::Ok().json(hourly),
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiError::internal_error("Failed to get hourly activity"))
        }
    }
}

#[get("/api/settings")]
pub async fn get_settings(state: web::Data<AppState>) -> impl Responder {
    let settings = state.settings.read().unwrap();
    HttpResponse::Ok().json(MonitorSettings {
        inactivity_seconds: settings.inactivity_seconds,
        sound_threshold: settings.sound_threshold,
    })
}

#[post("/api/settings")]
pub async fn update_settings(
    state: web::Data<AppState>,
    body: web::Json<MonitorSettings>,
) -> impl Responder {
    let mut settings = state.settings.write().unwrap();
    settings.inactivity_seconds = body.inactivity_seconds;
    settings.sound_threshold = body.sound_threshold;
    
    info!("Settings updated: inactivity={}s, sound_threshold={}", 
        settings.inactivity_seconds, settings.sound_threshold);
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "Settings updated successfully"
    }))
}
