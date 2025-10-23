use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use std::future::Future;
use std::pin::Pin;

pub type DynFuture = Pin<Box<dyn Future<Output = Zval> + 'static>>;

#[php_class]
pub struct RustFuture {
    inner: Option<DynFuture>,
}

impl RustFuture {
    pub fn new<F>(future: F) -> Self 
    where 
        F: Future + 'static,
        F::Output: IntoZval,
    {
        Self {
            inner: Some(Box::pin(async move {
                let res = future.await;
                res.into_zval(false).unwrap_or_else(|_| Zval::new()) 
            })),
        }
    }

    pub fn take_inner(&mut self) -> Option<DynFuture> {
        self.inner.take()
    }
}

#[php_impl]
impl RustFuture {
    // spawn removed, logic moved to global go() function
}
