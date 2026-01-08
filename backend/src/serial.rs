//! Serial communication module for Arduino

use chrono::Utc;
use serialport::SerialPortType;
use std::io::{BufRead, BufReader};
use std::sync::{mpsc::{self, Receiver, Sender}, Arc, RwLock};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::fhir::{AlertType, SensorEvent, SensorReading};
use crate::api::MonitorSettings;

#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub sound_threshold: i32,
    pub inactivity_seconds: u64,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            port: "COM3".to_string(),
            baud_rate: 9600,
            sound_threshold: 150,
            inactivity_seconds: 300,
        }
    }
}

pub fn list_available_ports() -> Vec<String> {
    match serialport::available_ports() {
        Ok(ports) => {
            let mut result = Vec::new();
            for port in ports {
                let port_type = match &port.port_type {
                    SerialPortType::UsbPort(info) => {
                        format!("USB - {}", info.manufacturer.as_deref().unwrap_or("Unknown"))
                    }
                    _ => "Unknown".to_string(),
                };
                info!("Found port: {} ({})", port.port_name, port_type);
                result.push(port.port_name);
            }
            result
        }
        Err(e) => {
            error!("Failed to list serial ports: {}", e);
            Vec::new()
        }
    }
}

pub struct SerialReader {
    receiver: Receiver<SensorEvent>,
    _handle: thread::JoinHandle<()>,
}

impl SerialReader {
    pub fn start(config: SerialConfig, settings: Arc<RwLock<MonitorSettings>>) -> Result<Self, String> {
        info!("Opening serial port: {} at {} baud", config.port, config.baud_rate);
        
        let (sender, receiver): (Sender<SensorEvent>, Receiver<SensorEvent>) = mpsc::channel();
        
        let port_name = config.port.clone();
        let baud_rate = config.baud_rate;
        
        let port = serialport::new(&port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open()
            .map_err(|e| format!("Failed to open {}: {}", port_name, e))?;
        
        info!("Serial port opened successfully");
        
        let handle = thread::spawn(move || {
            Self::read_loop(port, sender, config, settings);
        });
        
        Ok(Self {
            receiver,
            _handle: handle,
        })
    }
    
    fn read_loop(port: Box<dyn serialport::SerialPort>, sender: Sender<SensorEvent>, config: SerialConfig, settings: Arc<RwLock<MonitorSettings>>) {
        let mut reader = BufReader::new(port);
        let mut last_motion_time = std::time::Instant::now();
        let mut line_buffer = String::new();
        
        info!("Serial reader thread started");
        
        loop {
            line_buffer.clear();
            
            match reader.read_line(&mut line_buffer) {
                Ok(0) => {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Ok(_) => {
                    let line = line_buffer.trim();
                    
                    if line.is_empty() {
                        continue;
                    }
                    
                    debug!("Raw serial data: {}", line);
                    
                    match Self::parse_line(line) {
                        Some(reading) => {
                            if reading.motion {
                                last_motion_time = std::time::Instant::now();
                            }
                            
                            let alert = Self::detect_alert(
                                &reading,
                                &settings,
                                last_motion_time.elapsed().as_secs(),
                            );
                            
                            let event = SensorEvent {
                                id: None,
                                reading,
                                alert,
                            };
                            
                            if sender.send(event).is_err() {
                                break;
                            }
                        }
                        None => {
                            warn!("Failed to parse line: {}", line);
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::TimedOut {
                        error!("Serial read error: {}", e);
                    }
                }
            }
        }
        
        info!("Serial reader thread stopped");
    }
    
    fn parse_line(line: &str) -> Option<SensorReading> {
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() != 3 {
            return None;
        }
        
        let temperature = parts[0].trim().parse::<f32>().ok()?;
        let motion = parts[1].trim().parse::<i32>().ok()? != 0;
        let sound_level = parts[2].trim().parse::<i32>().ok()?;
        
        Some(SensorReading {
            temperature,
            motion,
            sound_level,
            timestamp: Utc::now(),
        })
    }
    
    fn detect_alert(reading: &SensorReading, settings: &Arc<RwLock<MonitorSettings>>, seconds_since_motion: u64) -> AlertType {
        let settings = settings.read().unwrap();
        
        if reading.motion && reading.sound_level > settings.sound_threshold {
            info!(">>> FALL ALERT: motion={}, sound={}", reading.motion, reading.sound_level);
            return AlertType::Fall;
        }
        
        if seconds_since_motion > settings.inactivity_seconds {
            info!(">>> INACTIVITY ALERT: no motion for {} seconds", seconds_since_motion);
            return AlertType::Inactivity;
        }
        
        AlertType::None
    }
    
    pub fn try_recv(&self) -> Option<SensorEvent> {
        self.receiver.try_recv().ok()
    }
}

/// Mock serial reader for testing without Arduino
pub struct MockSerialReader {
    receiver: Receiver<SensorEvent>,
    _handle: thread::JoinHandle<()>,
}

impl MockSerialReader {
    pub fn start() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        let handle = thread::spawn(move || {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            
            loop {
                let reading = SensorReading {
                    temperature: 20.0 + rng.r#gen::<f32>() * 10.0,
                    motion: rng.r#gen::<f32>() < 0.3,
                    sound_level: if rng.r#gen::<f32>() < 0.1 {
                        rng.gen_range(150..400)
                    } else {
                        rng.gen_range(10..50)
                    },
                    timestamp: Utc::now(),
                };
                
                let alert = if reading.motion && reading.sound_level > 150 {
                    AlertType::Fall
                } else {
                    AlertType::None
                };
                
                let event = SensorEvent {
                    id: None,
                    reading,
                    alert,
                };
                
                if sender.send(event).is_err() {
                    break;
                }
                
                thread::sleep(Duration::from_secs(1));
            }
        });
        
        Self {
            receiver,
            _handle: handle,
        }
    }
    
    pub fn try_recv(&self) -> Option<SensorEvent> {
        self.receiver.try_recv().ok()
    }
}