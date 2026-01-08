//! Smart Patient Room Monitor - Backend Server

mod api;
mod db;
mod fhir;
mod serial;
mod websocket;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::api::{AppState, MonitorSettings};
use crate::db::{Database, DbConfig};
use crate::serial::{SerialConfig, SerialReader};
use crate::websocket::SensorBroadcaster;

struct Config {
    host: String,
    port: u16,
    serial_port: String,
    baud_rate: u32,
    sound_threshold: i32,
    inactivity_seconds: u64,
    db_config: DbConfig,
    mock_mode: bool,
}

impl Config {
    fn from_env() -> Self {
        dotenvy::dotenv().ok();
        
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080),
            serial_port: std::env::var("SERIAL_PORT").unwrap_or_else(|_| "COM3".to_string()),
            baud_rate: std::env::var("BAUD_RATE").ok().and_then(|b| b.parse().ok()).unwrap_or(9600),
            sound_threshold: std::env::var("SOUND_THRESHOLD").ok().and_then(|s| s.parse().ok()).unwrap_or(150),
            inactivity_seconds: std::env::var("INACTIVITY_SECONDS").ok().and_then(|s| s.parse().ok()).unwrap_or(300),
            db_config: DbConfig::from_env(),
            mock_mode: std::env::var("MOCK_MODE").map(|v| v == "true" || v == "1").unwrap_or(false),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    
    info!("========================================");
    info!("  Smart Patient Room Monitor v0.1.0");
    info!("========================================");
    
    let config = Config::from_env();
    
    info!("Server: {}:{}", config.host, config.port);
    info!("Serial: {} @ {} baud", config.serial_port, config.baud_rate);
    info!("Mock mode: {}", config.mock_mode);
    
    // Initialize database
    let db = Database::new(config.db_config)
        .await
        .expect("Failed to initialize database");
    
    // Initialize broadcaster
    let broadcaster = Arc::new(SensorBroadcaster::new(100));
    
    // Initialize settings (shared between AppState and SerialReader)
    let settings = Arc::new(RwLock::new(MonitorSettings {
        inactivity_seconds: config.inactivity_seconds,
        sound_threshold: config.sound_threshold,
    }));
    
    // Start serial reader
    let serial_config = SerialConfig {
        port: config.serial_port.clone(),
        baud_rate: config.baud_rate,
        sound_threshold: config.sound_threshold,
        inactivity_seconds: config.inactivity_seconds,
    };
    
    let db_for_serial = db.clone();
    let broadcaster_for_serial = Arc::clone(&broadcaster);
    let settings_for_serial = Arc::clone(&settings);
    
    if config.mock_mode {
        info!("Starting in MOCK MODE");
        let mock_reader = serial::MockSerialReader::start();
        
        tokio::spawn(async move {
            loop {
                if let Some(mut event) = mock_reader.try_recv() {
                    match db_for_serial.insert_reading(&event).await {
                        Ok(id) => event.id = Some(id),
                        Err(e) => error!("Failed to save: {}", e),
                    }
                    broadcaster_for_serial.broadcast(event);
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    } else {
        info!("Available serial ports:");
        serial::list_available_ports();
        
        match SerialReader::start(serial_config, settings_for_serial) {
            Ok(reader) => {
                info!("Serial reader started");
                
                tokio::spawn(async move {
                    loop {
                        if let Some(mut event) = reader.try_recv() {
                            info!("Sensor: temp={:.1}°C motion={} sound={}",
                                event.reading.temperature,
                                event.reading.motion,
                                event.reading.sound_level);
                            
                            match db_for_serial.insert_reading(&event).await {
                                Ok(id) => event.id = Some(id),
                                Err(e) => error!("Failed to save: {}", e),
                            }
                            broadcaster_for_serial.broadcast(event);
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                });
            }
            Err(e) => {
                error!("Failed to start serial reader: {}", e);
                error!("Set MOCK_MODE=true to run without Arduino");
            }
        }
    }
    
    let app_state = web::Data::new(AppState {
        db: db.clone(),
        base_url: format!("http://{}:{}", config.host, config.port),
        settings: settings,
    });
    
    let broadcaster_data = web::Data::new(broadcaster);
    
    info!("Starting server on {}:{}", config.host, config.port);
    info!("Dashboard: http://{}:{}", config.host, config.port);
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        
        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .app_data(broadcaster_data.clone())
            .service(api::health_check)
            .service(api::list_observations)
            .service(api::get_latest_observation)
            .service(api::get_observation_by_id)
            .service(api::get_summary)
            .service(api::get_sleep_analysis)
            .service(api::get_period_analysis)
            .service(api::get_hourly_analysis)
            .service(api::get_settings)
            .service(api::update_settings)
            .route("/ws", web::get().to(websocket::ws_handler))
            .service(actix_files::Files::new("/", "./frontend").index_file("index.html"))
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}
