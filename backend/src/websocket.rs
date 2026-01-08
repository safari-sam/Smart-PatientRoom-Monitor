//! WebSocket module for real-time data streaming

use actix_web::{rt, web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

use crate::fhir::{AlertType, SensorEvent};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsMessage {
    #[serde(rename_all = "camelCase")]
    SensorReading {
        temperature: f32,
        motion: bool,
        sound_level: i32,
        timestamp: String,
        alert: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Status {
        connected: bool,
        message: String,
    },
    Ping {
        timestamp: String,
    },
}

impl From<&SensorEvent> for WsMessage {
    fn from(event: &SensorEvent) -> Self {
        WsMessage::SensorReading {
            temperature: event.reading.temperature,
            motion: event.reading.motion,
            sound_level: event.reading.sound_level,
            timestamp: event.reading.timestamp.to_rfc3339(),
            alert: match event.alert {
                AlertType::None => None,
                AlertType::Fall => Some("FALL_DETECTED".to_string()),
                AlertType::Inactivity => Some("INACTIVITY_ALERT".to_string()),
            },
        }
    }
}

#[derive(Clone)]
pub struct SensorBroadcaster {
    sender: broadcast::Sender<SensorEvent>,
}

impl SensorBroadcaster {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<SensorEvent> {
        self.sender.subscribe()
    }
    
    pub fn broadcast(&self, event: SensorEvent) {
        let _ = self.sender.send(event);
    }
}

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    broadcaster: web::Data<Arc<SensorBroadcaster>>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut stream) = actix_ws::handle(&req, stream)?;
    
    info!("New WebSocket connection established");
    
    let mut rx = broadcaster.subscribe();
    
    let welcome = WsMessage::Status {
        connected: true,
        message: "Connected to Smart Patient Monitor".to_string(),
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = session.text(json).await;
    }
    
    rt::spawn(async move {
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            tokio::select! {
                Some(msg) = stream.recv() => {
                    match msg {
                        Ok(Message::Ping(bytes)) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            info!("WebSocket closed");
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
                
                Ok(event) = rx.recv() => {
                    let msg = WsMessage::from(&event);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if session.text(json).await.is_err() {
                            break;
                        }
                    }
                }
                
                _ = heartbeat_interval.tick() => {
                    let ping = WsMessage::Ping {
                        timestamp: Utc::now().to_rfc3339(),
                    };
                    if let Ok(json) = serde_json::to_string(&ping) {
                        if session.text(json).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
        
        let _ = session.close(None).await;
    });
    
    Ok(response)
}