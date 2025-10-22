use ext_php_rs::prelude::*;
use crate::future::RustFuture;
use ext_php_rs::types::Zval;
use std::time::{SystemTime, UNIX_EPOCH};

#[php_class]
pub struct AsyncTime;

#[php_impl]
impl AsyncTime {
    /// Pauses the current fiber for at least the duration ms (in milliseconds).
    pub fn sleep(ms: i64) -> RustFuture {
        let future = async move {
            tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
            Zval::new()
        };
        RustFuture::new(future)
    }

    /// Returns the current time as a Unix timestamp in milliseconds.
    pub fn now() -> i64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis() as i64
    }
}
