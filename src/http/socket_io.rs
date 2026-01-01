use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::zend::ClassEntry;
use ext_php_rs::convert::IntoZval;
use socketioxide::SocketIo;
use socketioxide::extract::{SocketRef, Data};
use socketioxide::layer::SocketIoLayer;
use std::sync::{Arc, Mutex};
use crate::util::{zval_to_json, json_to_zval};

struct UnsafeSendZval(Zval);
unsafe impl Send for UnsafeSendZval {}
unsafe impl Sync for UnsafeSendZval {}

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\SocketIo\\Socket")]
pub struct AsyncSocket {
    inner: SocketRef,
}

#[php_impl]
impl AsyncSocket {
    #[php]
    pub fn id(&self) -> String {
        self.inner.id.to_string()
    }

    #[php]
    pub fn emit(&self, event: String, data: Option<&Zval>) -> PhpResult<()> {
        let val = if let Some(d) = data {
            zval_to_json(d)
        } else {
            serde_json::Value::Null
        };
        
        self.inner.emit(event, &val).map_err(|e| {
            ext_php_rs::exception::PhpException::default(format!("Emit error: {}", e))
        })?;
        Ok(())
    }
    
    #[php]
    pub fn join(&self, room: String) -> PhpResult<()> {
        self.inner.join(room);
        Ok(())
    }

    #[php]
    pub fn leave(&self, room: String) -> PhpResult<()> {
        self.inner.leave(room);
        Ok(())
    }

    #[php]
    pub fn broadcast(&self, event: String, data: Option<&Zval>) -> PhpResult<()> {
        let val = if let Some(d) = data {
            zval_to_json(d)
        } else {
            serde_json::Value::Null
        };
        
        let socket = self.inner.clone();
        crate::runtime::context::spawn_local(async move {
            if let Err(e) = socket.broadcast().emit(event, &val).await {
                eprintln!("Broadcast error: {}", e);
            }
        });
        Ok(())
    }

    #[php]
    pub fn to(&self, room: String, event: String, data: Option<&Zval>) -> PhpResult<()> {
        let val = if let Some(d) = data {
            zval_to_json(d)
        } else {
            serde_json::Value::Null
        };
        
        let socket = self.inner.clone();
        crate::runtime::context::spawn_local(async move {
            if let Err(e) = socket.to(room).emit(event, &val).await {
                 eprintln!("Emit to room error: {}", e);
            }
        });
        Ok(())
    }

    #[php]
    pub fn on(&self, event: String, callback: &Zval) -> PhpResult<()> {
        let cb = Arc::new(Mutex::new(UnsafeSendZval(callback.shallow_clone())));
        
        self.inner.on(event, {
            let cb = cb.clone();
            move |socket: SocketRef, data: Data<serde_json::Value>| {
                let cb = cb.clone();
                async move {
                    crate::runtime::context::spawn_local(async move {
                        // Logic to spawn fiber
                        let fiber_class = match ClassEntry::try_find("Fiber") {
                            Some(ce) => ce,
                            None => {
                                eprintln!("Fiber class not found");
                                return;
                            }
                        };

                        let fiber_obj = fiber_class.new();
                        let fiber_zval = match fiber_obj.into_zval(false) {
                            Ok(z) => z,
                            Err(e) => {
                                eprintln!("Failed to convert fiber: {:?}", e);
                                return;
                            }
                        };

                        let callback_zval = cb.lock().unwrap().0.shallow_clone();

                        if let Err(e) = fiber_zval.try_call_method("__construct", vec![&callback_zval])
                        {
                            eprintln!("Failed to construct Fiber: {:?}", e);
                            return;
                        }

                        let fiber_clone = fiber_zval.shallow_clone();

                        let socket_obj = AsyncSocket { inner: socket };
                        let socket_zval = ext_php_rs::types::ZendClassObject::new(socket_obj)
                            .into_zval(false)
                            .unwrap_or_else(|_| Zval::new());

                        let data_zval = json_to_zval(&data.0);

                        let args: Vec<&dyn ext_php_rs::convert::IntoZvalDyn> =
                            vec![&data_zval, &socket_zval];
                        if let Err(e) = crate::runtime::runtime::drive_fiber(fiber_clone, args).await {
                            eprintln!("Event handler fiber failed: {:?}", e);
                        }
                    });
                }
            }
        });
        Ok(())
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\SocketIo")]
pub struct AsyncSocketIo {
    pub(crate) layer: Option<SocketIoLayer>,
    io: SocketIo,
}

#[php_impl]
impl AsyncSocketIo {
    #[php]
    pub fn __construct() -> Self {
        let (layer, io) = SocketIo::new_layer();
        Self {
            layer: Some(layer),
            io,
        }
    }

    #[php]
    pub fn on_connection(&mut self, namespace: String, callback: &Zval) -> PhpResult<()> {
        let cb = Arc::new(Mutex::new(UnsafeSendZval(callback.shallow_clone())));
        
        self.io.ns(namespace, {
            let cb = cb.clone();
            move |socket: SocketRef| {
                let cb = cb.clone();
                async move {
                    crate::runtime::context::spawn_local(async move {
                        // Logic to spawn fiber
                        let fiber_class = match ClassEntry::try_find("Fiber") {
                            Some(ce) => ce,
                            None => {
                                eprintln!("Fiber class not found");
                                return;
                            }
                        };

                        let fiber_obj = fiber_class.new();
                        let fiber_zval = match fiber_obj.into_zval(false) {
                            Ok(z) => z,
                            Err(e) => {
                                eprintln!("Failed to convert fiber: {:?}", e);
                                return;
                            }
                        };

                        let callback_zval = cb.lock().unwrap().0.shallow_clone();

                        if let Err(e) = fiber_zval.try_call_method("__construct", vec![&callback_zval])
                        {
                            eprintln!("Failed to construct Fiber: {:?}", e);
                            return;
                        }

                        let fiber_clone = fiber_zval.shallow_clone();

                        let socket_obj = AsyncSocket { inner: socket };
                        let socket_zval = ext_php_rs::types::ZendClassObject::new(socket_obj)
                            .into_zval(false)
                            .unwrap_or_else(|_| Zval::new());

                        let args: Vec<&dyn ext_php_rs::convert::IntoZvalDyn> = vec![&socket_zval];
                        if let Err(e) = crate::runtime::runtime::drive_fiber(fiber_clone, args).await {
                            eprintln!("Socket.IO connection handler fiber error: {:?}", e);
                        }
                    });
                }
            }
        });
        Ok(())
    }
}
