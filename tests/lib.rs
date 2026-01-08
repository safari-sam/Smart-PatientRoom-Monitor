//! Smart Patient Room Monitor - Test Suite
//! 
//! This crate contains all unit and integration tests for the Smart Patient Monitor.
//! 
//! ## Test Categories
//! 
//! - **fhir_tests**: Tests for FHIR data structures and serialization
//! - **alert_tests**: Tests for fall detection and inactivity alert logic
//! - **api_tests**: Tests for REST API endpoints and responses
//! - **activity_tests**: Tests for activity analysis and sleep scoring
//! - **db_tests**: Tests for database CRUD operations
//! 
//! ## Running Tests
//! 
//! ```bash
//! # Run all tests
//! cargo test
//! 
//! # Run tests with output
//! cargo test -- --nocapture
//! 
//! # Run specific test module
//! cargo test fhir
//! cargo test alert
//! cargo test api
//! cargo test activity
//! cargo test db
//! 
//! # Run specific test
//! cargo test test_fall_detected
//! ```
//! 
//! ## Test Coverage
//! 
//! | Module | Tests | Coverage |
//! |--------|-------|----------|
//! | FHIR Structures | 8 | Data models, serialization |
//! | Alert Detection | 15 | Fall detection, inactivity |
//! | API Endpoints | 20 | Health, observations, bundles |
//! | Activity Analysis | 20 | Scoring, levels, quality |
//! | Database | 18 | CRUD operations, summaries |

// Include test modules
mod fhir_tests;
mod alert_tests;
mod api_tests;
mod activity_tests;
mod db_tests;

// Re-export for documentation
pub use fhir_tests::*;
pub use alert_tests::*;
pub use api_tests::*;
pub use activity_tests::*;
pub use db_tests::*;
