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
    /// Pauses the current fiber for at least the duration ms (in milliseconds).
    pub fn sleep(ms: i64) -> RustFuture {
        let future = async move {
            tokio_sleep(Duration::from_millis(ms as u64)).await;
            Zval::new()
        };
        RustFuture::new(future)
    }

    /// Returns a RustFuture that resolves after the duration ms (in milliseconds).
    pub fn after(ms: i64) -> RustFuture {
        Self::sleep(ms)
    }

    /// Schedules a callback to be executed after seconds.
    pub fn timer(seconds: f64, callback: &Zval) {
        let callback = callback.shallow_clone();
        let ms = (seconds * 1000.0) as u64;

        crate::context::spawn_local(async move {
            tokio_sleep(Duration::from_millis(ms)).await;
            if let Err(e) = callback.try_call(vec![]) {
                eprintln!("Timer callback failed: {}", e);
            }
        });
    }

    /// Returns the current time as a Unix timestamp in milliseconds.
    pub fn now() -> i64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis() as i64
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
