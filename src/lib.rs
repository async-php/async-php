use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::exception::PhpException;

mod future;
mod net;
mod fs;
mod http;
mod channel;
mod util;

use future::RustFuture;
use net::{AsyncTcpListener, AsyncTcpStream, AsyncUdpSocket};
use fs::{AsyncFilesystem, AsyncFileHandle};
use http::{HttpParser, HttpReq};
use channel::AsyncChannel;

pub(crate) async fn drive_fiber(fiber: Zval) -> PhpResult<()> {
    let mut current_val = fiber
        .try_call_method("start", vec![])
        .map_err(|e| PhpException::default(format!("Fiber start error: {}", e)))?;

    loop {
        let is_terminated = fiber
            .try_call_method("isTerminated", vec![])
            .map_err(|e| PhpException::default(format!("Check terminated error: {}", e)))?
            .bool()
            .unwrap_or(false);

        if is_terminated {
            break;
        }

        if current_val.is_null() {
             tokio::task::yield_now().await;
             current_val = fiber
                .try_call_method("resume", vec![])
                .map_err(|e| PhpException::default(format!("Fiber resume error: {}", e)))?;
             continue;
        }

        if let Some(rust_fut) = <&mut RustFuture as ext_php_rs::convert::FromZvalMut>::from_zval_mut(&mut current_val) {
            if let Some(fut) = rust_fut.take_inner() {
                let result = fut.await;
                let args: Vec<&dyn ext_php_rs::convert::IntoZvalDyn> = vec![&result];
                current_val = fiber
                    .try_call_method("resume", args)
                    .map_err(|e| PhpException::default(format!("Fiber resume error: {}", e)))?;
            } else {
                return Err(PhpException::default("RustFuture was already awaited!".into()));
            }
        } else {
            return Err(PhpException::default(format!("Fiber suspended with unknown value. Type: {:?}", current_val.get_type())));
        }
    }
    Ok(())
}

#[php_function]
pub fn run(fiber: &mut Zval) -> PhpResult<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| PhpException::default(format!("Tokio Error: {}", e)))?;

    let local = tokio::task::LocalSet::new();

    local.block_on(&rt, async {
        let fiber_clone = fiber.shallow_clone();
        drive_fiber(fiber_clone).await
    })
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<RustFuture>()
        .class::<AsyncTcpListener>()
        .class::<AsyncTcpStream>()
        .class::<AsyncUdpSocket>()
        .class::<AsyncFilesystem>()
        .class::<AsyncFileHandle>()
        .class::<HttpParser>()
        .class::<HttpReq>()
        .class::<AsyncChannel>()
        .function(wrap_function!(run))
}
