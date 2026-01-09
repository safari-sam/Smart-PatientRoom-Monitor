//! FHIR-compliant data models for patient monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// CORE SENSOR DATA
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub temperature: f32,
    pub motion: bool,
    pub sound_level: i32,  // Integer for sound level
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertType {
    None,
    Fall,
    Inactivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorEvent {
    pub id: Option<i64>,
    pub reading: SensorReading,
    pub alert: AlertType,
}

// ============================================================================
// FHIR STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirCoding {
    pub system: String,
    pub code: String,
    pub display: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirCodeableConcept {
    pub coding: Vec<FhirCoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirQuantity {
    pub value: f64,
    pub unit: String,
    pub system: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirReference {
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirObservationComponent {
    pub code: FhirCodeableConcept,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_quantity: Option<FhirQuantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_boolean: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_integer: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirObservation {
    pub resource_type: String,
    pub id: String,
    pub status: String,
    pub category: Vec<FhirCodeableConcept>,
    pub code: FhirCodeableConcept,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<FhirReference>,
    pub effective_date_time: String,
    pub issued: String,
    pub component: Vec<FhirObservationComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<Vec<FhirCodeableConcept>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirBundleEntry {
    pub full_url: String,
    pub resource: FhirObservation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirBundle {
    pub resource_type: String,
    pub id: String,
    #[serde(rename = "type")]
    pub bundle_type: String,
    pub total: u32,
    pub timestamp: String,
    pub entry: Vec<FhirBundleEntry>,
}

// ============================================================================
// CONVERSION IMPLEMENTATIONS
// ============================================================================

impl SensorEvent {
    pub fn to_fhir(&self, base_url: &str) -> FhirObservation {
        let obs_id = self.id
            .map(|id| format!("observation-{}", id))
            .unwrap_or_else(|| format!("observation-{}", Uuid::new_v4()));
        
        let timestamp = self.reading.timestamp.to_rfc3339();
        
        let mut components = vec![
            FhirObservationComponent {
                code: FhirCodeableConcept {
                    coding: vec![FhirCoding {
                        system: "http://loinc.org".to_string(),
                        code: "8310-5".to_string(),
                        display: "Body temperature".to_string(),
                    }],
                    text: Some("Room Temperature".to_string()),
                },
                value_quantity: Some(FhirQuantity {
                    value: self.reading.temperature as f64,
                    unit: "Cel".to_string(),
                    system: "http://unitsofmeasure.org".to_string(),
                    code: "Cel".to_string(),
                }),
                value_boolean: None,
                value_integer: None,
                value_string: None,
            },
            FhirObservationComponent {
                code: FhirCodeableConcept {
                    coding: vec![FhirCoding {
                        system: "http://snomed.info/sct".to_string(),
                        code: "52821000".to_string(),
                        display: "Motion detected".to_string(),
                    }],
                    text: Some("Motion Sensor".to_string()),
                },
                value_quantity: None,
                value_boolean: Some(self.reading.motion),
                value_integer: None,
                value_string: None,
            },
            FhirObservationComponent {
                code: FhirCodeableConcept {
                    coding: vec![FhirCoding {
                        system: "http://loinc.org".to_string(),
                        code: "89020-2".to_string(),
                        display: "Sound level".to_string(),
                    }],
                    text: Some("Ambient Sound Level".to_string()),
                },
                value_quantity: None,
                value_boolean: None,
                value_integer: Some(self.reading.sound_level),
                value_string: None,
            },
        ];
        
        if self.alert != AlertType::None {
            components.push(FhirObservationComponent {
                code: FhirCodeableConcept {
                    coding: vec![FhirCoding {
                        system: "http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation".to_string(),
                        code: "AA".to_string(),
                        display: "Critical abnormal".to_string(),
                    }],
                    text: Some("Alert Status".to_string()),
                },
                value_quantity: None,
                value_boolean: None,
                value_integer: None,
                value_string: Some(match self.alert {
                    AlertType::Fall => "FALL_DETECTED".to_string(),
                    AlertType::Inactivity => "INACTIVITY_ALERT".to_string(),
                    AlertType::None => "NORMAL".to_string(),
                }),
            });
        }
        
        let interpretation = if self.alert != AlertType::None {
            Some(vec![FhirCodeableConcept {
                coding: vec![FhirCoding {
                    system: "http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation".to_string(),
                    code: "AA".to_string(),
                    display: "Critical abnormal".to_string(),
                }],
                text: Some(match self.alert {
                    AlertType::Fall => "Possible fall detected".to_string(),
                    AlertType::Inactivity => "Patient inactivity alert".to_string(),
                    AlertType::None => "Normal".to_string(),
                }),
            }])
        } else {
            None
        };
        
        FhirObservation {
            resource_type: "Observation".to_string(),
            id: obs_id,
            status: "final".to_string(),
            category: vec![FhirCodeableConcept {
                coding: vec![FhirCoding {
                    system: "http://terminology.hl7.org/CodeSystem/observation-category".to_string(),
                    code: "vital-signs".to_string(),
                    display: "Vital Signs".to_string(),
                }],
                text: None,
            }],
            code: FhirCodeableConcept {
                coding: vec![FhirCoding {
                    system: "http://loinc.org".to_string(),
                    code: "85353-1".to_string(),
                    display: "Vital signs panel".to_string(),
                }],
                text: Some("Patient Room Monitoring Panel".to_string()),
            },
            subject: Some(FhirReference {
                reference: "Patient/room-101".to_string(),
                display: Some("Room 101 Occupant".to_string()),
            }),
            effective_date_time: timestamp.clone(),
            issued: timestamp,
            component: components,
            interpretation,
        }
    }
}

impl FhirBundle {
    pub fn from_events(events: Vec<SensorEvent>, base_url: &str) -> Self {
        let entries: Vec<FhirBundleEntry> = events
            .iter()
            .map(|event| {
                let obs = event.to_fhir(base_url);
                FhirBundleEntry {
                    full_url: format!("{}/Observation/{}", base_url, obs.id),
                    resource: obs,
                }
            })
            .collect();
        
        FhirBundle {
            resource_type: "Bundle".to_string(),
            id: Uuid::new_v4().to_string(),
            bundle_type: "searchset".to_string(),
            total: entries.len() as u32,
            timestamp: Utc::now().to_rfc3339(),
            entry: entries,
        }
    }
}