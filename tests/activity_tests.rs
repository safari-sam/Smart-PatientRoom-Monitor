//! Unit tests for activity analysis functionality
//! 
//! These tests verify that activity scoring and sleep analysis work correctly.

#[cfg(test)]
mod tests {
    
    // ========================================================================
    // ACTIVITY ANALYSIS LOGIC
    // ========================================================================
    
    #[derive(Debug, Clone, PartialEq)]
    pub enum ActivityLevel {
        DeepSleep,
        LightSleep,
        Restless,
        Active,
    }
    
    /// Calculate activity score as percentage
    fn calculate_activity_score(motion_readings: u64, total_readings: u64) -> f64 {
        if total_readings == 0 {
            return 0.0;
        }
        (motion_readings as f64 / total_readings as f64) * 100.0
    }
    
    /// Determine activity level based on score
    fn get_activity_level(score: f64) -> ActivityLevel {
        match score {
            s if s < 20.0 => ActivityLevel::DeepSleep,
            s if s < 40.0 => ActivityLevel::LightSleep,
            s if s < 60.0 => ActivityLevel::Restless,
            _ => ActivityLevel::Active,
        }
    }
    
    /// Get rest quality description
    fn get_rest_quality(score: f64) -> &'static str {
        match score {
            s if s < 20.0 => "Excellent",
            s if s < 40.0 => "Good",
            s if s < 60.0 => "Fair",
            _ => "Poor",
        }
    }
    
    // ========================================================================
    // ACTIVITY SCORE TESTS
    // ========================================================================
    
    #[test]
    fn test_activity_score_zero_motion() {
        let score = calculate_activity_score(0, 100);
        assert_eq!(score, 0.0);
    }
    
    #[test]
    fn test_activity_score_all_motion() {
        let score = calculate_activity_score(100, 100);
        assert_eq!(score, 100.0);
    }
    
    #[test]
    fn test_activity_score_half_motion() {
        let score = calculate_activity_score(50, 100);
        assert_eq!(score, 50.0);
    }
    
    #[test]
    fn test_activity_score_quarter_motion() {
        let score = calculate_activity_score(25, 100);
        assert_eq!(score, 25.0);
    }
    
    #[test]
    fn test_activity_score_zero_readings() {
        let score = calculate_activity_score(0, 0);
        assert_eq!(score, 0.0);  // Should handle division by zero
    }
    
    #[test]
    fn test_activity_score_more_motion_than_total() {
        // Edge case: shouldn't happen but should handle gracefully
        let score = calculate_activity_score(150, 100);
        assert_eq!(score, 150.0);
    }
    
    #[test]
    fn test_activity_score_precision() {
        let score = calculate_activity_score(33, 100);
        assert_eq!(score, 33.0);
        
        let score2 = calculate_activity_score(1, 3);
        assert!((score2 - 33.333).abs() < 0.01);
    }
    
    // ========================================================================
    // ACTIVITY LEVEL TESTS
    // ========================================================================
    
    #[test]
    fn test_deep_sleep_level() {
        assert_eq!(get_activity_level(0.0), ActivityLevel::DeepSleep);
        assert_eq!(get_activity_level(10.0), ActivityLevel::DeepSleep);
        assert_eq!(get_activity_level(19.9), ActivityLevel::DeepSleep);
    }
    
    #[test]
    fn test_light_sleep_level() {
        assert_eq!(get_activity_level(20.0), ActivityLevel::LightSleep);
        assert_eq!(get_activity_level(30.0), ActivityLevel::LightSleep);
        assert_eq!(get_activity_level(39.9), ActivityLevel::LightSleep);
    }
    
    #[test]
    fn test_restless_level() {
        assert_eq!(get_activity_level(40.0), ActivityLevel::Restless);
        assert_eq!(get_activity_level(50.0), ActivityLevel::Restless);
        assert_eq!(get_activity_level(59.9), ActivityLevel::Restless);
    }
    
    #[test]
    fn test_active_level() {
        assert_eq!(get_activity_level(60.0), ActivityLevel::Active);
        assert_eq!(get_activity_level(80.0), ActivityLevel::Active);
        assert_eq!(get_activity_level(100.0), ActivityLevel::Active);
    }
    
    #[test]
    fn test_activity_level_boundaries() {
        // Test exact boundaries
        assert_eq!(get_activity_level(19.999), ActivityLevel::DeepSleep);
        assert_eq!(get_activity_level(20.0), ActivityLevel::LightSleep);
        assert_eq!(get_activity_level(39.999), ActivityLevel::LightSleep);
        assert_eq!(get_activity_level(40.0), ActivityLevel::Restless);
        assert_eq!(get_activity_level(59.999), ActivityLevel::Restless);
        assert_eq!(get_activity_level(60.0), ActivityLevel::Active);
    }
    
    // ========================================================================
    // REST QUALITY TESTS
    // ========================================================================
    
    #[test]
    fn test_excellent_rest_quality() {
        assert_eq!(get_rest_quality(0.0), "Excellent");
        assert_eq!(get_rest_quality(15.0), "Excellent");
    }
    
    #[test]
    fn test_good_rest_quality() {
        assert_eq!(get_rest_quality(20.0), "Good");
        assert_eq!(get_rest_quality(35.0), "Good");
    }
    
    #[test]
    fn test_fair_rest_quality() {
        assert_eq!(get_rest_quality(40.0), "Fair");
        assert_eq!(get_rest_quality(55.0), "Fair");
    }
    
    #[test]
    fn test_poor_rest_quality() {
        assert_eq!(get_rest_quality(60.0), "Poor");
        assert_eq!(get_rest_quality(100.0), "Poor");
    }
    
    // ========================================================================
    // INTEGRATION TESTS - Full Analysis Flow
    // ========================================================================
    
    #[test]
    fn test_full_analysis_deep_sleep() {
        let motion_readings = 10;
        let total_readings = 100;
        
        let score = calculate_activity_score(motion_readings, total_readings);
        let level = get_activity_level(score);
        let quality = get_rest_quality(score);
        
        assert_eq!(score, 10.0);
        assert_eq!(level, ActivityLevel::DeepSleep);
        assert_eq!(quality, "Excellent");
    }
    
    #[test]
    fn test_full_analysis_restless_night() {
        let motion_readings = 45;
        let total_readings = 100;
        
        let score = calculate_activity_score(motion_readings, total_readings);
        let level = get_activity_level(score);
        let quality = get_rest_quality(score);
        
        assert_eq!(score, 45.0);
        assert_eq!(level, ActivityLevel::Restless);
        assert_eq!(quality, "Fair");
    }
    
    #[test]
    fn test_full_analysis_active_patient() {
        let motion_readings = 75;
        let total_readings = 100;
        
        let score = calculate_activity_score(motion_readings, total_readings);
        let level = get_activity_level(score);
        let quality = get_rest_quality(score);
        
        assert_eq!(score, 75.0);
        assert_eq!(level, ActivityLevel::Active);
        assert_eq!(quality, "Poor");
    }
    
    // ========================================================================
    // EDGE CASES
    // ========================================================================
    
    #[test]
    fn test_single_reading_with_motion() {
        let score = calculate_activity_score(1, 1);
        assert_eq!(score, 100.0);
        assert_eq!(get_activity_level(score), ActivityLevel::Active);
    }
    
    #[test]
    fn test_single_reading_without_motion() {
        let score = calculate_activity_score(0, 1);
        assert_eq!(score, 0.0);
        assert_eq!(get_activity_level(score), ActivityLevel::DeepSleep);
    }
    
    #[test]
    fn test_large_number_of_readings() {
        let score = calculate_activity_score(5000, 10000);
        assert_eq!(score, 50.0);
        assert_eq!(get_activity_level(score), ActivityLevel::Restless);
    }
}
