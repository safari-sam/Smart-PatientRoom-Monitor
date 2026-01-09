//! Database module for PostgreSQL

use chrono::{DateTime, Utc};
use deadpool_postgres::{Config, Pool, Runtime, ManagerConfig, RecyclingMethod};
use tokio_postgres::{NoTls, Row};
use tracing::{info, debug};

use crate::fhir::{AlertType, SensorEvent, SensorReading};

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

impl DbConfig {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DB_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            user: std::env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string()),
            dbname: std::env::var("DB_NAME").unwrap_or_else(|_| "patient_monitor".to_string()),
        }
    }
}

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn new(config: DbConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Connecting to PostgreSQL at {}:{}", config.host, config.port);
        
        let mut cfg = Config::new();
        cfg.host = Some(config.host);
        cfg.port = Some(config.port);
        cfg.user = Some(config.user);
        cfg.password = Some(config.password);
        cfg.dbname = Some(config.dbname);
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        
        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
        
        let db = Self { pool };
        db.init_schema().await?;
        
        info!("Database initialized successfully");
        Ok(db)
    }
    
    async fn init_schema(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        
        client.execute(
            "CREATE TABLE IF NOT EXISTS sensor_data (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                temperature REAL NOT NULL,
                motion BOOLEAN NOT NULL,
                sound_level INTEGER NOT NULL,
                alert_type VARCHAR(20) NOT NULL DEFAULT 'none'
            )",
            &[],
        ).await?;
        
        client.execute(
            "CREATE INDEX IF NOT EXISTS idx_sensor_timestamp ON sensor_data(timestamp DESC)",
            &[],
        ).await?;
        
        Ok(())
    }
    
    pub async fn insert_reading(&self, event: &SensorEvent) -> Result<i64, Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        
        let alert_str = match event.alert {
            AlertType::None => "none",
            AlertType::Fall => "fall",
            AlertType::Inactivity => "inactivity",
        };
        
        let row = client.query_one(
            "INSERT INTO sensor_data (timestamp, temperature, motion, sound_level, alert_type)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
            &[
                &event.reading.timestamp,
                &event.reading.temperature,
                &event.reading.motion,
                &event.reading.sound_level,
                &alert_str,
            ],
        ).await?;
        
        let id: i64 = row.get(0);
        debug!("Inserted reading with ID: {}", id);
        
        Ok(id)
    }
    
    pub async fn get_recent_readings(&self, limit: usize) -> Result<Vec<SensorEvent>, Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        
        let rows = client.query(
            "SELECT id, timestamp, temperature, motion, sound_level, alert_type
             FROM sensor_data
             ORDER BY timestamp DESC
             LIMIT $1",
            &[&(limit as i64)],
        ).await?;
        
        let events = rows.iter().map(Self::row_to_event).collect();
        Ok(events)
    }
    
    pub async fn get_readings_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<SensorEvent>, Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        
        let rows = client.query(
            "SELECT id, timestamp, temperature, motion, sound_level, alert_type
             FROM sensor_data
             WHERE timestamp BETWEEN $1 AND $2
             ORDER BY timestamp DESC",
            &[&start, &end],
        ).await?;
        
        let events = rows.iter().map(Self::row_to_event).collect();
        Ok(events)
    }
    
    pub async fn get_reading_by_id(&self, id: i64) -> Result<Option<SensorEvent>, Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        
        let row = client.query_opt(
            "SELECT id, timestamp, temperature, motion, sound_level, alert_type
             FROM sensor_data WHERE id = $1",
            &[&id],
        ).await?;
        
        Ok(row.map(|r| Self::row_to_event(&r)))
    }
    
    pub async fn get_alert_summary(&self) -> Result<AlertSummary, Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        
        let total: i64 = client.query_one("SELECT COUNT(*) FROM sensor_data", &[])
            .await?.get(0);
        
        let falls: i64 = client.query_one(
            "SELECT COUNT(*) FROM sensor_data WHERE alert_type = 'fall'", &[]
        ).await?.get(0);
        
        let inactivity: i64 = client.query_one(
            "SELECT COUNT(*) FROM sensor_data WHERE alert_type = 'inactivity'", &[]
        ).await?.get(0);
        
        Ok(AlertSummary {
            total_readings: total as u64,
            fall_alerts: falls as u64,
            inactivity_alerts: inactivity as u64,
        })
    }
    
    fn row_to_event(row: &Row) -> SensorEvent {
        let id: i64 = row.get(0);
        let timestamp: DateTime<Utc> = row.get(1);
        let temperature: f32 = row.get(2);
        let motion: bool = row.get(3);
        let sound_level: i32 = row.get(4);
        let alert_str: &str = row.get(5);
        
        let alert = match alert_str {
            "fall" => AlertType::Fall,
            "inactivity" => AlertType::Inactivity,
            _ => AlertType::None,
        };
        
        SensorEvent {
            id: Some(id),
            reading: SensorReading {
                temperature,
                motion,
                sound_level,
                timestamp,
            },
            alert,
        }
    }
    
    /// Analyze patient activity for a specific time period
    pub async fn get_activity_analysis(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<ActivityAnalysis, Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        
        // Get aggregate statistics
        let stats_row = client.query_one(
            "SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE motion = true) as motion_count,
                COALESCE(AVG(temperature), 0.0::float) as avg_temp,
                COALESCE(AVG(sound_level), 0.0::float) as avg_sound,
                COALESCE(MAX(sound_level), 0) as max_sound,
                COUNT(*) FILTER (WHERE alert_type = 'fall') as falls
             FROM sensor_data 
             WHERE timestamp BETWEEN $1 AND $2",
            &[&start, &end],
        ).await?;
        
        let total: i64 = stats_row.get(0);
        let motion_count: i64 = stats_row.get(1);
        let avg_temp: f64 = stats_row.get(2);
        let avg_sound: f64 = stats_row.get(3);
        let max_sound: i32 = stats_row.get(4);
        let falls: i64 = stats_row.get(5);
        
        // Calculate activity score (0-100)
        let activity_score = if total > 0 {
            (motion_count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        // Determine activity level
        let activity_level = match activity_score {
            s if s < 20.0 => "deep_sleep",
            s if s < 40.0 => "light_sleep", 
            s if s < 60.0 => "restless",
            _ => "active",
        }.to_string();
        
        // Calculate longest still period
        let longest_still = self.calculate_longest_still_period(start, end).await?;
        
        Ok(ActivityAnalysis {
            period_start: start.to_rfc3339(),
            period_end: end.to_rfc3339(),
            total_readings: total as u64,
            motion_readings: motion_count as u64,
            activity_score: (activity_score * 100.0).round() / 100.0,
            activity_level,
            avg_temperature: (avg_temp * 100.0).round() / 100.0,
            avg_sound_level: (avg_sound * 100.0).round() / 100.0,
            max_sound_level: max_sound,
            fall_alerts: falls as u64,
            longest_still_period_mins: longest_still,
        })
    }
    
    async fn calculate_longest_still_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        
        let rows = client.query(
            "SELECT timestamp, motion FROM sensor_data 
             WHERE timestamp BETWEEN $1 AND $2 
             ORDER BY timestamp ASC",
            &[&start, &end],
        ).await?;
        
        if rows.is_empty() {
            return Ok(0);
        }
        
        let mut longest_still: i64 = 0;
        let mut current_still_start: Option<DateTime<Utc>> = None;
        
        for row in &rows {
            let timestamp: DateTime<Utc> = row.get(0);
            let motion: bool = row.get(1);
            
            if !motion {
                if current_still_start.is_none() {
                    current_still_start = Some(timestamp);
                }
            } else {
                if let Some(start_time) = current_still_start {
                    let duration = timestamp.signed_duration_since(start_time).num_minutes();
                    if duration > longest_still {
                        longest_still = duration;
                    }
                    current_still_start = None;
                }
            }
        }
        
        if let Some(start_time) = current_still_start {
            let duration = end.signed_duration_since(start_time).num_minutes();
            if duration > longest_still {
                longest_still = duration;
            }
        }
        
        Ok(longest_still as u64)
    }
    
    /// Get hourly activity breakdown
    pub async fn get_hourly_activity(
        &self,
        date: DateTime<Utc>,
    ) -> Result<Vec<HourlyActivity>, Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        
        let rows = client.query(
            "SELECT 
                DATE_TRUNC('hour', timestamp) as hour,
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE motion = true) as motion_count,
                COALESCE(AVG(sound_level), 0.0::float) as avg_sound
             FROM sensor_data 
             WHERE timestamp::date = $1::date
             GROUP BY DATE_TRUNC('hour', timestamp)
             ORDER BY hour",
            &[&date],
        ).await?;
        
        let mut hourly = Vec::new();
        for row in rows {
            let hour: DateTime<Utc> = row.get(0);
            let total: i64 = row.get(1);
            let motion_count: i64 = row.get(2);
            let avg_sound: f64 = row.get(3);
            
            let activity_score = if total > 0 {
                (motion_count as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            hourly.push(HourlyActivity {
                hour: hour.format("%H:00").to_string(),
                activity_score: (activity_score * 100.0).round() / 100.0,
                readings: total as u64,
                avg_sound_level: (avg_sound * 100.0).round() / 100.0,
            });
        }
        
        Ok(hourly)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AlertSummary {
    pub total_readings: u64,
    pub fall_alerts: u64,
    pub inactivity_alerts: u64,
}

/// Activity analysis for a time period
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityAnalysis {
    pub period_start: String,
    pub period_end: String,
    pub total_readings: u64,
    pub motion_readings: u64,
    pub activity_score: f64,
    pub activity_level: String,
    pub avg_temperature: f64,
    pub avg_sound_level: f64,
    pub max_sound_level: i32,
    pub fall_alerts: u64,
    pub longest_still_period_mins: u64,
}

/// Hourly activity breakdown
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyActivity {
    pub hour: String,
    pub activity_score: f64,
    pub readings: u64,
    pub avg_sound_level: f64,
}
