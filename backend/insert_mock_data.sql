-- Mock data for testing report generation
-- This script inserts 7 days of sensor data with various patterns

-- Day 1: Normal day with regular activity
INSERT INTO sensor_data (timestamp, temperature, motion, sound_level, alert_type) VALUES
('2026-01-01 06:00:00+00', 22.5, false, 3, 'none'),
('2026-01-01 07:30:00+00', 23.2, true, 5, 'none'),
('2026-01-01 08:00:00+00', 23.8, true, 7, 'none'),
('2026-01-01 09:15:00+00', 24.1, true, 6, 'none'),
('2026-01-01 10:30:00+00', 24.5, false, 4, 'none'),
('2026-01-01 12:00:00+00', 25.0, true, 8, 'none'),
('2026-01-01 13:30:00+00', 25.3, true, 7, 'none'),
('2026-01-01 15:00:00+00', 25.1, false, 5, 'none'),
('2026-01-01 17:00:00+00', 24.8, true, 6, 'none'),
('2026-01-01 19:00:00+00', 24.2, true, 9, 'none'),
('2026-01-01 21:00:00+00', 23.5, false, 4, 'none'),
('2026-01-01 23:00:00+00', 22.8, false, 2, 'none');

-- Day 2: Day with one fall alert
INSERT INTO sensor_data (timestamp, temperature, motion, sound_level, alert_type) VALUES
('2026-01-02 06:00:00+00', 22.3, false, 3, 'none'),
('2026-01-02 08:00:00+00', 23.5, true, 6, 'none'),
('2026-01-02 10:00:00+00', 24.2, true, 7, 'none'),
('2026-01-02 11:30:00+00', 24.8, true, 8, 'none'),
('2026-01-02 14:00:00+00', 25.2, false, 12, 'fall'),  -- Fall detected
('2026-01-02 14:30:00+00', 25.1, true, 7, 'none'),
('2026-01-02 16:00:00+00', 24.9, true, 6, 'none'),
('2026-01-02 18:00:00+00', 24.5, true, 8, 'none'),
('2026-01-02 20:00:00+00', 23.8, false, 5, 'none'),
('2026-01-02 22:00:00+00', 23.1, false, 3, 'none');

-- Day 3: High temperature day
INSERT INTO sensor_data (timestamp, temperature, motion, sound_level, alert_type) VALUES
('2026-01-03 06:00:00+00', 23.0, false, 3, 'none'),
('2026-01-03 08:00:00+00', 24.5, true, 6, 'none'),
('2026-01-03 10:00:00+00', 26.2, true, 7, 'none'),
('2026-01-03 12:00:00+00', 27.5, true, 8, 'none'),
('2026-01-03 14:00:00+00', 28.1, false, 6, 'none'),
('2026-01-03 16:00:00+00', 27.8, true, 7, 'none'),
('2026-01-03 18:00:00+00', 26.5, true, 8, 'none'),
('2026-01-03 20:00:00+00', 25.2, false, 5, 'none'),
('2026-01-03 22:00:00+00', 24.1, false, 4, 'none');

-- Day 4: Low activity day (inactivity alert)
INSERT INTO sensor_data (timestamp, temperature, motion, sound_level, alert_type) VALUES
('2026-01-04 06:00:00+00', 22.8, false, 2, 'none'),
('2026-01-04 08:00:00+00', 23.2, false, 3, 'none'),
('2026-01-04 10:00:00+00', 23.8, false, 2, 'none'),
('2026-01-04 12:00:00+00', 24.1, false, 3, 'inactivity'),  -- Extended inactivity
('2026-01-04 14:00:00+00', 24.3, false, 2, 'inactivity'),
('2026-01-04 16:00:00+00', 24.0, true, 5, 'none'),  -- Activity resumes
('2026-01-04 18:00:00+00', 23.5, true, 6, 'none'),
('2026-01-04 20:00:00+00', 23.0, false, 4, 'none'),
('2026-01-04 22:00:00+00', 22.5, false, 3, 'none');

-- Day 5: Very active day with high sound levels
INSERT INTO sensor_data (timestamp, temperature, motion, sound_level, alert_type) VALUES
('2026-01-05 06:00:00+00', 22.5, true, 7, 'none'),
('2026-01-05 07:00:00+00', 23.1, true, 8, 'none'),
('2026-01-05 08:00:00+00', 23.8, true, 9, 'none'),
('2026-01-05 09:00:00+00', 24.2, true, 8, 'none'),
('2026-01-05 10:00:00+00', 24.7, true, 10, 'none'),
('2026-01-05 11:00:00+00', 25.0, true, 9, 'none'),
('2026-01-05 12:00:00+00', 25.3, true, 11, 'none'),
('2026-01-05 14:00:00+00', 25.5, true, 10, 'none'),
('2026-01-05 16:00:00+00', 25.2, true, 9, 'none'),
('2026-01-05 18:00:00+00', 24.8, true, 8, 'none'),
('2026-01-05 20:00:00+00', 24.1, true, 7, 'none'),
('2026-01-05 22:00:00+00', 23.4, false, 5, 'none');

-- Day 6: Mixed pattern with cold morning
INSERT INTO sensor_data (timestamp, temperature, motion, sound_level, alert_type) VALUES
('2026-01-06 06:00:00+00', 20.5, false, 2, 'none'),
('2026-01-06 07:00:00+00', 21.2, false, 3, 'none'),
('2026-01-06 08:00:00+00', 21.8, true, 5, 'none'),
('2026-01-06 09:00:00+00', 22.5, true, 6, 'none'),
('2026-01-06 10:00:00+00', 23.2, true, 7, 'none'),
('2026-01-06 11:00:00+00', 23.8, false, 4, 'none'),
('2026-01-06 12:00:00+00', 24.3, true, 6, 'none'),
('2026-01-06 13:00:00+00', 24.8, true, 7, 'none'),
('2026-01-06 14:00:00+00', 25.1, false, 5, 'none'),
('2026-01-06 16:00:00+00', 24.7, true, 6, 'none'),
('2026-01-06 18:00:00+00', 24.2, true, 8, 'none'),
('2026-01-06 20:00:00+00', 23.5, false, 5, 'none'),
('2026-01-06 22:00:00+00', 22.8, false, 3, 'none');

-- Day 7: Recent day (today) - ongoing data
INSERT INTO sensor_data (timestamp, temperature, motion, sound_level, alert_type) VALUES
('2026-01-06 00:00:00+00', 22.0, false, 2, 'none'),
('2026-01-06 02:00:00+00', 21.8, false, 2, 'none'),
('2026-01-06 04:00:00+00', 21.5, false, 1, 'none'),
('2026-01-06 06:00:00+00', 21.8, false, 3, 'none'),
('2026-01-06 08:00:00+00', 22.5, true, 6, 'none'),
('2026-01-06 10:00:00+00', 23.5, true, 7, 'none'),
('2026-01-06 12:00:00+00', 24.2, true, 8, 'none');

-- Additional granular data for the last 24 hours (every hour)
INSERT INTO sensor_data (timestamp, temperature, motion, sound_level, alert_type) VALUES
('2026-01-05 13:00:00+00', 24.8, true, 7, 'none'),
('2026-01-05 14:00:00+00', 25.1, true, 8, 'none'),
('2026-01-05 15:00:00+00', 25.3, false, 6, 'none'),
('2026-01-05 16:00:00+00', 25.0, true, 7, 'none'),
('2026-01-05 17:00:00+00', 24.7, true, 8, 'none'),
('2026-01-05 18:00:00+00', 24.3, true, 9, 'none'),
('2026-01-05 19:00:00+00', 23.9, true, 7, 'none'),
('2026-01-05 20:00:00+00', 23.5, false, 5, 'none'),
('2026-01-05 21:00:00+00', 23.1, false, 4, 'none'),
('2026-01-05 22:00:00+00', 22.8, false, 3, 'none'),
('2026-01-05 23:00:00+00', 22.5, false, 2, 'none');

SELECT COUNT(*) as total_records FROM sensor_data;
SELECT alert_type, COUNT(*) as count FROM sensor_data GROUP BY alert_type;
SELECT DATE(timestamp) as date, COUNT(*) as records_per_day FROM sensor_data GROUP BY DATE(timestamp) ORDER BY date;
