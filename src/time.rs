use ext_php_rs::prelude::*;
use crate::future::RustFuture;
use ext_php_rs::types::Zval;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio::time::sleep as tokio_sleep;

#[php_class]
#[php(name = "Async\\Kernel\\Time")]
pub struct AsyncTime;

#[php_impl]
impl AsyncTime {
    /// Pauses the current fiber for the given number of seconds (can be fractional).
    ///
    /// # Arguments
    /// * `seconds` - Time to sleep in seconds.
    pub fn sleep(seconds: f64) -> RustFuture {
        let future = async move {
            tokio_sleep(Duration::from_secs_f64(seconds)).await;
            Zval::new()
        };
        RustFuture::new(future)
    }

    /// Pauses the current fiber for the given number of microseconds.
    pub fn usleep(micros: i64) -> RustFuture {
        let future = async move {
            tokio_sleep(Duration::from_micros(micros as u64)).await;
            Zval::new()
        };
        RustFuture::new(future)
    }

    /// Wraps a Future with a timeout.
    ///
    /// If the future completes before the timeout, its result is returned.
    /// If the timeout elapses, an exception is thrown.
    pub fn timeout(seconds: f64, future: &mut RustFuture) -> RustFuture {
        let duration = Duration::from_secs_f64(seconds);
        // Take the inner future from the passed RustFuture wrapper
        let inner = future.take_inner();

        RustFuture::new(async move {
            if let Some(fut) = inner {
                match tokio::time::timeout(duration, fut).await {
                    Ok(result) => result, // Future completed successfully
                    Err(_) => Err("Operation timed out".to_string()),
                }
            } else {
                Err("Invalid future (already consumed)".to_string())
            }
        })
    }

    /// Schedules a callback to be executed after seconds.
    pub fn timer(seconds: f64, callback: &Zval) {
        let callback = callback.shallow_clone();
        let duration = Duration::from_secs_f64(seconds);

        crate::runtime::context::spawn_local(async move {
            tokio_sleep(duration).await;
            if let Err(e) = callback.try_call(vec![]) {
                eprintln!("Timer callback failed: {}", e);
            }
        });
    }

    /// Returns the current time as a Unix timestamp with microsecond precision (float).
    /// Similar to PHP's microtime(true).
    pub fn now() -> f64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_secs_f64()
    }

    /// Creates a new Ticker that sends messages on a channel at intervals.
    ///
    /// # Arguments
    /// * `interval_ms` - The interval in milliseconds between ticks
    pub fn create_ticker(interval_ms: i64) -> PhpResult<AsyncTicker> {
        let (tx, _rx) = broadcast::channel(1);
        let tx_clone = tx.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(interval_ms as u64));
            interval.tick().await; // First tick fires immediately, skip it
            loop {
                interval.tick().await;
                if tx_clone.send(()).is_err() {
                    break;
                }
            }
        });

        let state = TickerState {
            tx: Some(tx),
            handle: Some(handle),
            interval_ms,
            tick_count: 0,
            max_ticks: None,
            is_paused: false,
        };

        Ok(AsyncTicker {
            state: Arc::new(Mutex::new(state)),
        })
    }
}

struct TickerState {
    tx: Option<broadcast::Sender<()>>,
    handle: Option<JoinHandle<()>>,
    interval_ms: i64,
    tick_count: u64,
    max_ticks: Option<u64>,
    is_paused: bool,
}

#[php_class]
#[php(name = "Async\\Kernel\\Ticker")]
pub struct AsyncTicker {
    state: Arc<Mutex<TickerState>>,
}

#[php_impl]
impl AsyncTicker {
    /// Wait for the next tick
    pub fn next_tick(&self) -> PhpResult<RustFuture> {
        let state_clone = Arc::clone(&self.state);

        let state = self.state.lock().unwrap();
        if let Some(tx) = &state.tx {
            let mut rx = tx.subscribe();
            let future = async move {
                let _ = rx.recv().await;

                // Increment tick count
                let mut state = state_clone.lock().unwrap();
                state.tick_count += 1;

                // Check if we've reached max ticks
                if let Some(max) = state.max_ticks {
                    if state.tick_count >= max {
                        // Auto-stop when reaching max ticks
                        if let Some(handle) = state.handle.take() {
                            handle.abort();
                        }
                        state.tx = None;
                    }
                }

                Zval::new()
            };
            Ok(RustFuture::new(future))
        } else {
            Err(ext_php_rs::exception::PhpException::default("Ticker has been stopped.".into()))
        }
    }

    /// Stop the ticker permanently
    pub fn stop(&self) {
        let mut state = self.state.lock().unwrap();
        if let Some(handle) = state.handle.take() {
            handle.abort();
        }
        state.tx = None;
    }

    /// Reset the ticker (restart from beginning)
    pub fn reset(&self) -> PhpResult<()> {
        let mut state = self.state.lock().unwrap();

        // Stop existing ticker
        if let Some(handle) = state.handle.take() {
            handle.abort();
        }

        // Create new ticker
        let (tx, _rx) = broadcast::channel(1);
        let tx_clone = tx.clone();
        let interval_ms = state.interval_ms;

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(interval_ms as u64));
            interval.tick().await; // First tick fires immediately, skip it
            loop {
                interval.tick().await;
                if tx_clone.send(()).is_err() {
                    break;
                }
            }
        });

        state.tx = Some(tx);
        state.handle = Some(handle);
        state.tick_count = 0;
        state.is_paused = false;

        Ok(())
    }

    /// Get the interval in milliseconds
    pub fn get_interval(&self) -> i64 {
        let state = self.state.lock().unwrap();
        state.interval_ms
    }

    /// Get the current tick count
    pub fn get_tick_count(&self) -> u64 {
        let state = self.state.lock().unwrap();
        state.tick_count
    }

    /// Set maximum number of ticks (0 or None means unlimited)
    pub fn set_max_ticks(&self, max: Option<i64>) {
        let mut state = self.state.lock().unwrap();
        state.max_ticks = max.filter(|&m| m > 0).map(|m| m as u64);
    }

    /// Get the maximum number of ticks
    pub fn get_max_ticks(&self) -> Option<i64> {
        let state = self.state.lock().unwrap();
        state.max_ticks.map(|m| m as i64)
    }

    /// Check if the ticker is running
    pub fn is_running(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.tx.is_some() && !state.is_paused
    }

    /// Check if the ticker is stopped
    pub fn is_stopped(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.tx.is_none()
    }
}
