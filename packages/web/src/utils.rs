//! Utility functions for the web package

/// Format seconds to HH:mm:ss format
pub fn format_time(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

/// Parse HH:mm:ss format to seconds
pub fn parse_time(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    let hours = parts[0].parse::<u64>().ok()?;
    let minutes = parts[1].parse::<u64>().ok()?;
    let seconds = parts[2].parse::<u64>().ok()?;

    Some((hours * 3600 + minutes * 60 + seconds) as f64)
}
