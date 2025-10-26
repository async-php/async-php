use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use std::future::Future;
use std::pin::Pin;

pub type DynFuture = Pin<Box<dyn Future<Output = Result<Zval, String>> + 'static>>;

pub trait ToRustFutureResult {
    fn to_result(self) -> Result<Zval, String>;
}

impl<T, E> ToRustFutureResult for Result<T, E>
where
    T: IntoZval,
    E: ToString,
{
    fn to_result(self) -> Result<Zval, String> {
        match self {
            Ok(v) => v.into_zval(false).map_err(|e| format!("Zval conversion error: {:?}", e)),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl ToRustFutureResult for Zval {
    fn to_result(self) -> Result<Zval, String> {
        Ok(self)
    }
}

impl ToRustFutureResult for () {
    fn to_result(self) -> Result<Zval, String> {
        Ok(Zval::new())
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\RustFuture")]
pub struct RustFuture {
    inner: Option<DynFuture>,
}

impl RustFuture {
    pub fn new<F, O>(future: F) -> Self 
    where 
        F: Future<Output = O> + 'static,
        O: ToRustFutureResult,
    {
        Self {
            inner: Some(Box::pin(async move {
                future.await.to_result()
            })),
        }
    }

    pub fn take_inner(&mut self) -> Option<DynFuture> {
        self.inner.take()
    }
}

