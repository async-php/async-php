/// HTTP request performance metrics
use std::time::{Duration, Instant};
use ext_php_rs::prelude::*;

/// Performance metrics for HTTP requests
#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\RequestMetrics")]
#[derive(Clone, Debug)]
pub struct RequestMetrics {
    /// Total request duration from start to completion
    total_time: f64,
    /// Time to first byte (TTFB) - time until response headers received
    time_to_first_byte: f64,
    /// Time spent sending the request
    request_send_time: f64,
    /// Number of redirects followed
    redirect_count: u32,
}

#[php_impl]
impl RequestMetrics {
    /// Get total request time in seconds
    #[php]
    pub fn get_total_time(&self) -> f64 {
        self.total_time
    }

    /// Get time to first byte in seconds
    #[php]
    pub fn get_time_to_first_byte(&self) -> f64 {
        self.time_to_first_byte
    }

    /// Get request send time in seconds
    #[php]
    pub fn get_request_send_time(&self) -> f64 {
        self.request_send_time
    }

    /// Get number of redirects followed
    #[php]
    pub fn get_redirect_count(&self) -> u32 {
        self.redirect_count
    }

    /// Get metrics as associative array
    #[php]
    pub fn to_array(&self) -> Vec<(String, f64)> {
        vec![
            ("total_time".to_string(), self.total_time),
            ("time_to_first_byte".to_string(), self.time_to_first_byte),
            ("request_send_time".to_string(), self.request_send_time),
            ("redirect_count".to_string(), self.redirect_count as f64),
        ]
    }
}

/// Internal metrics collector for tracking request timing
#[derive(Clone)]
pub struct MetricsCollector {
    start_time: Instant,
    request_sent_time: Option<Instant>,
    first_byte_time: Option<Instant>,
    redirect_count: u32,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            request_sent_time: None,
            first_byte_time: None,
            redirect_count: 0,
        }
    }

    /// Mark when the request was sent
    pub fn mark_request_sent(&mut self) {
        self.request_sent_time = Some(Instant::now());
    }

    /// Mark when first byte was received
    pub fn mark_first_byte(&mut self) {
        if self.first_byte_time.is_none() {
            self.first_byte_time = Some(Instant::now());
        }
    }

    /// Increment redirect counter
    pub fn increment_redirects(&mut self) {
        self.redirect_count += 1;
    }

    /// Build final metrics
    pub fn build(self) -> RequestMetrics {
        let end_time = Instant::now();
        let total_time = (end_time - self.start_time).as_secs_f64();

        let time_to_first_byte = self.first_byte_time
            .map(|t| (t - self.start_time).as_secs_f64())
            .unwrap_or(total_time);

        let request_send_time = self.request_sent_time
            .map(|t| (t - self.start_time).as_secs_f64())
            .unwrap_or(0.0);

        RequestMetrics {
            total_time,
            time_to_first_byte,
            request_send_time,
            redirect_count: self.redirect_count,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        sleep(Duration::from_millis(10));
        collector.mark_request_sent();

        sleep(Duration::from_millis(10));
        collector.mark_first_byte();

        collector.increment_redirects();

        let metrics = collector.build();

        assert!(metrics.total_time > 0.0);
        assert!(metrics.time_to_first_byte > 0.0);
        assert!(metrics.request_send_time > 0.0);
        assert_eq!(metrics.redirect_count, 1);
    }

    #[test]
    fn test_metrics_to_array() {
        let metrics = RequestMetrics {
            total_time: 1.5,
            time_to_first_byte: 0.5,
            request_send_time: 0.1,
            redirect_count: 2,
        };

        let array = metrics.to_array();
        assert_eq!(array.len(), 4);
        assert_eq!(array[0].0, "total_time");
        assert_eq!(array[0].1, 1.5);
    }
}
