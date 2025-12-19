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

        crate::context::spawn_local(async move {
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
        };

        Ok(AsyncTicker {
            state: Arc::new(Mutex::new(state)),
        })
    }
}

struct TickerState {
    tx: Option<broadcast::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

#[php_class]
#[php(name = "Async\\Kernel\\Ticker")]
pub struct AsyncTicker {
    state: Arc<Mutex<TickerState>>,
}

#[php_impl]
impl AsyncTicker {
    pub fn next_tick(&self) -> PhpResult<RustFuture> {
        let state = self.state.lock().unwrap();
        if let Some(tx) = &state.tx {
            let mut rx = tx.subscribe();
            let future = async move {
                let _ = rx.recv().await;
                Zval::new()
            };
            Ok(RustFuture::new(future))
        } else {
            Err(ext_php_rs::exception::PhpException::default("Ticker has been stopped.".into()))
        }
    }

    pub fn stop(&self) {
        let mut state = self.state.lock().unwrap();
        if let Some(handle) = state.handle.take() {
            handle.abort();
        }
        state.tx = None;
    }
}
