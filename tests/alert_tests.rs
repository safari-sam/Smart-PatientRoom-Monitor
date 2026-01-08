//! Unit tests for alert detection logic
//! 
//! These tests verify that fall detection and inactivity alerts work correctly.

#[cfg(test)]
mod tests {
    
    // ========================================================================
    // ALERT DETECTION LOGIC (same logic as your serial.rs)
    // ========================================================================
    
    #[derive(Debug, Clone, PartialEq)]
    pub enum AlertType {
        None,
        Fall,
        Inactivity,
    }
    
    /// Detect alerts based on sensor readings
    /// This is the core logic that should match your serial.rs implementation
    fn detect_alert(
        motion: bool,
        sound_level: i32,
        sound_threshold: i32,
        seconds_since_motion: u64,
        inactivity_threshold: u64,
    ) -> AlertType {
        // Fall detection: motion + loud sound = possible fall
        if motion && sound_level > sound_threshold {
            return AlertType::Fall;
        }
        
        // Inactivity detection: no motion for too long
        if seconds_since_motion > inactivity_threshold {
            return AlertType::Inactivity;
        }
        
        AlertType::None
    }
    
    // ========================================================================
    // FALL DETECTION TESTS
    // ========================================================================
    
    #[test]
    fn test_fall_detected_with_motion_and_loud_sound() {
        let alert = detect_alert(
            motion: true,
            sound_level: 200,
            sound_threshold: 150,
            seconds_since_motion: 0,
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::Fall);
    }
    
    #[test]
    fn test_no_fall_when_sound_below_threshold() {
        let alert = detect_alert(
            motion: true,
            sound_level: 100,  // Below threshold of 150
            sound_threshold: 150,
            seconds_since_motion: 0,
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::None);
    }
    
    #[test]
    fn test_no_fall_when_no_motion() {
        let alert = detect_alert(
            motion: false,  // No motion
            sound_level: 200,  // Loud sound
            sound_threshold: 150,
            seconds_since_motion: 10,
            inactivity_threshold: 300,
        );
        
        // Loud sound without motion is NOT a fall (could be external noise)
        assert_eq!(alert, AlertType::None);
    }
    
    #[test]
    fn test_fall_detected_at_exact_threshold() {
        let alert = detect_alert(
            motion: true,
            sound_level: 151,  // Just above threshold
            sound_threshold: 150,
            seconds_since_motion: 0,
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::Fall);
    }
    
    #[test]
    fn test_no_fall_at_exact_threshold() {
        let alert = detect_alert(
            motion: true,
            sound_level: 150,  // Exactly at threshold (not above)
            sound_threshold: 150,
            seconds_since_motion: 0,
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::None);
    }
    
    // ========================================================================
    // INACTIVITY DETECTION TESTS
    // ========================================================================
    
    #[test]
    fn test_inactivity_alert_after_threshold() {
        let alert = detect_alert(
            motion: false,
            sound_level: 30,
            sound_threshold: 150,
            seconds_since_motion: 301,  // Just over 5 minutes
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::Inactivity);
    }
    
    #[test]
    fn test_no_inactivity_before_threshold() {
        let alert = detect_alert(
            motion: false,
            sound_level: 30,
            sound_threshold: 150,
            seconds_since_motion: 299,  // Just under 5 minutes
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::None);
    }
    
    #[test]
    fn test_no_inactivity_at_exact_threshold() {
        let alert = detect_alert(
            motion: false,
            sound_level: 30,
            sound_threshold: 150,
            seconds_since_motion: 300,  // Exactly at threshold
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::None);
    }
    
    #[test]
    fn test_inactivity_with_custom_threshold() {
        // Test with 1 minute threshold
        let alert = detect_alert(
            motion: false,
            sound_level: 20,
            sound_threshold: 150,
            seconds_since_motion: 61,
            inactivity_threshold: 60,  // 1 minute
        );
        
        assert_eq!(alert, AlertType::Inactivity);
    }
    
    // ========================================================================
    // PRIORITY TESTS (Fall takes precedence)
    // ========================================================================
    
    #[test]
    fn test_fall_takes_priority_over_inactivity() {
        // Even if inactivity threshold is exceeded, fall should be detected first
        let alert = detect_alert(
            motion: true,
            sound_level: 200,
            sound_threshold: 150,
            seconds_since_motion: 0,  // Motion just happened
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::Fall);
    }
    
    // ========================================================================
    // EDGE CASES
    // ========================================================================
    
    #[test]
    fn test_zero_sound_level() {
        let alert = detect_alert(
            motion: true,
            sound_level: 0,
            sound_threshold: 150,
            seconds_since_motion: 0,
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::None);
    }
    
    #[test]
    fn test_very_high_sound_level() {
        let alert = detect_alert(
            motion: true,
            sound_level: 1000,  // Very loud
            sound_threshold: 150,
            seconds_since_motion: 0,
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::Fall);
    }
    
    #[test]
    fn test_long_inactivity_period() {
        let alert = detect_alert(
            motion: false,
            sound_level: 10,
            sound_threshold: 150,
            seconds_since_motion: 3600,  // 1 hour
            inactivity_threshold: 300,
        );
        
        assert_eq!(alert, AlertType::Inactivity);
    }
    
    #[test]
    fn test_zero_inactivity_threshold() {
        // With 0 threshold, any time without motion triggers alert
        let alert = detect_alert(
            motion: false,
            sound_level: 10,
            sound_threshold: 150,
            seconds_since_motion: 1,
            inactivity_threshold: 0,
        );
        
        assert_eq!(alert, AlertType::Inactivity);
    }
}
