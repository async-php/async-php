use ext_php_rs::php_class;
use ext_php_rs::php_impl;
use ext_php_rs::exception::PhpResult;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use socketioxide::SocketIo;
use socketioxide::extract::{SocketRef, Data};
use socketioxide::layer::SocketIoLayer;
use socketioxide::socket::DisconnectReason;
use std::sync::{Arc, Mutex};
use crate::util::{zval_to_json, json_to_zval};
use crate::runtime::runtime::{call_closure_async, call_method_async};

pub struct UnsafeSendZval(pub Zval);
unsafe impl Send for UnsafeSendZval {}
unsafe impl Sync for UnsafeSendZval {}

impl Clone for UnsafeSendZval {
    fn clone(&self) -> Self {
        Self(self.0.shallow_clone())
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\SocketIo\\Socket")]
pub struct AsyncSocket {
    inner: SocketRef,
    adapter: Option<UnsafeSendZval>,
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
        self.inner.join(room.clone());
        
        if let Some(adapter) = &self.adapter {
             let adapter = adapter.0.shallow_clone();
             let id = self.inner.id.to_string();
             crate::runtime::context::spawn_local(async move {
                 let id_zval = id.into_zval(false).unwrap_or(Zval::new());
                 let room_zval = room.into_zval(false).unwrap_or(Zval::new());
                 let _ = call_method_async(adapter, "add", vec![id_zval, room_zval]).await;
             });
        }
        Ok(())
    }

    #[php]
    pub fn leave(&self, room: String) -> PhpResult<()> {
        self.inner.leave(room.clone());
        
        if let Some(adapter) = &self.adapter {
             let adapter = adapter.0.shallow_clone();
             let id = self.inner.id.to_string();
             crate::runtime::context::spawn_local(async move {
                 let id_zval = id.into_zval(false).unwrap_or(Zval::new());
                 let room_zval = room.into_zval(false).unwrap_or(Zval::new());
                 let _ = call_method_async(adapter, "del", vec![id_zval, room_zval]).await;
             });
        }
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
        let event_clone = event.clone();
        
        // Local broadcast
        crate::runtime::context::spawn_local(async move {
            if let Err(e) = socket.broadcast().emit(event_clone, &val).await {
                eprintln!("Broadcast error: {}", e);
            }
        });

        // Remote broadcast hook
        if let Some(adapter) = &self.adapter {
             let adapter = adapter.0.shallow_clone();
             let id = self.inner.id.to_string();
             
             // Construct packet
             let mut packet = ext_php_rs::types::ZendHashTable::new();
             let _ = packet.push(event.into_zval(false).unwrap_or(Zval::new()));
             if let Some(d) = data {
                 let _ = packet.push(d.shallow_clone());
             }
             let packet_zval = packet.into_zval(false).unwrap_or(Zval::new());
             
             // Construct opts
             let mut opts = ext_php_rs::types::ZendHashTable::new();
             let mut except = ext_php_rs::types::ZendHashTable::new();
             let _ = except.push(id.into_zval(false).unwrap_or(Zval::new()));
             let _ = opts.insert("except", except.into_zval(false).unwrap_or(Zval::new()));
             let opts_zval = opts.into_zval(false).unwrap_or(Zval::new());
             
             crate::runtime::context::spawn_local(async move {
                 let _ = call_method_async(adapter, "broadcast", vec![packet_zval, opts_zval]).await;
             });
        }

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
        let room_clone = room.clone();
        let event_clone = event.clone();
        
        // Local emit to room
        crate::runtime::context::spawn_local(async move {
            if let Err(e) = socket.to(room_clone).emit(event_clone, &val).await {
                 eprintln!("Emit to room error: {}", e);
            }
        });
        
        // Remote broadcast hook
        if let Some(adapter) = &self.adapter {
             let adapter = adapter.0.shallow_clone();
             let id = self.inner.id.to_string();
             
             // Packet
             let mut packet = ext_php_rs::types::ZendHashTable::new();
             let _ = packet.push(event.into_zval(false).unwrap_or(Zval::new()));
             if let Some(d) = data {
                 let _ = packet.push(d.shallow_clone());
             }
             let packet_zval = packet.into_zval(false).unwrap_or(Zval::new());
             
             // Opts
             let mut opts = ext_php_rs::types::ZendHashTable::new();
             
             let mut rooms_ht = ext_php_rs::types::ZendHashTable::new();
             let _ = rooms_ht.push(room.into_zval(false).unwrap_or(Zval::new()));
             let _ = opts.insert("rooms", rooms_ht.into_zval(false).unwrap_or(Zval::new()));
             
             let mut except = ext_php_rs::types::ZendHashTable::new();
             let _ = except.push(id.into_zval(false).unwrap_or(Zval::new()));
             let _ = opts.insert("except", except.into_zval(false).unwrap_or(Zval::new()));
             
             let opts_zval = opts.into_zval(false).unwrap_or(Zval::new());
             
             crate::runtime::context::spawn_local(async move {
                 let _ = call_method_async(adapter, "broadcast", vec![packet_zval, opts_zval]).await;
             });
        }
        
        Ok(())
    }

    #[php]
    pub fn on(&self, event: String, callback: &Zval) -> PhpResult<()> {
        let cb = Arc::new(Mutex::new(UnsafeSendZval(callback.shallow_clone())));
        let adapter = self.adapter.clone();
        
        self.inner.on(event, {
            let cb = cb.clone();
            move |socket: SocketRef, data: Data<serde_json::Value>| {
                let cb = cb.clone();
                let adapter = adapter.clone();
                async move {
                    crate::runtime::context::spawn_local(async move {
                        let callback_zval = cb.lock().unwrap().0.shallow_clone();
                        let socket_obj = AsyncSocket { inner: socket, adapter: adapter.clone() };
                        let socket_zval = ext_php_rs::types::ZendClassObject::new(socket_obj)
                            .into_zval(false)
                            .unwrap_or_else(|_| Zval::new());

                        let data_zval = json_to_zval(&data.0);

                        let args = vec![data_zval, socket_zval];
                        
                        if let Err(e) = call_closure_async(callback_zval, args).await {
                             eprintln!("Event handler error: {:?}", e);
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
    adapter: Option<UnsafeSendZval>,
}

#[php_impl]
impl AsyncSocketIo {
    #[php]
    pub fn __construct(adapter: Option<&Zval>) -> Self {
        let (layer, io) = SocketIo::new_layer();
        Self {
            layer: Some(layer),
            io,
            adapter: adapter.map(|z| UnsafeSendZval(z.shallow_clone())),
        }
    }

    #[php]
    pub fn on_connection(&mut self, namespace: String, callback: &Zval) -> PhpResult<()> {
        let cb = Arc::new(Mutex::new(UnsafeSendZval(callback.shallow_clone())));
        let adapter = self.adapter.clone();
        
        self.io.ns(namespace, {
            let cb = cb.clone();
            move |socket: SocketRef| {
                let cb = cb.clone();
                let adapter_for_closure = adapter.clone();
                
                async move {
                    crate::runtime::context::spawn_local(async move {
                        let callback_zval = cb.lock().unwrap().0.shallow_clone();
                        let socket_obj = AsyncSocket { inner: socket.clone(), adapter: adapter_for_closure.clone() };
                        let socket_zval = ext_php_rs::types::ZendClassObject::new(socket_obj)
                            .into_zval(false)
                            .unwrap_or_else(|_| Zval::new());

                        let args = vec![socket_zval];
                        
                        if let Err(e) = call_closure_async(callback_zval, args).await {
                             eprintln!("Connection handler error: {:?}", e);
                        }
                        
                        // Hook: disconnect
                        if let Some(adapter) = adapter_for_closure {
                            let id = socket.id.to_string();
                            socket.on_disconnect(move || {
                                let adapter = adapter.clone(); 
                                async move {
                                    crate::runtime::context::spawn_local(async move {
                                        let adapter = adapter.0.shallow_clone();
                                        let id_zval = id.into_zval(false).unwrap_or(Zval::new());
                                        let _ = call_method_async(adapter, "delAll", vec![id_zval]).await;
                                    });
                                }
                            });
                        }
                    });
                }
            }
        });
        Ok(())
    }

    #[php]
    pub fn publish(&self, packet: &Zval, opts: &Zval) -> PhpResult<()> {
        // Parse packet: [event, data]
        let packet_ht = packet.array().ok_or_else(|| ext_php_rs::exception::PhpException::default("Packet must be an array".into()))?;
        let event = packet_ht.get_index(0).ok_or_else(|| ext_php_rs::exception::PhpException::default("Packet must have event".into()))?.str().unwrap_or_default();
        let data = packet_ht.get_index(1).map(|d| zval_to_json(d)).unwrap_or(serde_json::Value::Null);

        // Parse opts: { rooms: [], except: [] }
        let opts_ht = opts.array().ok_or_else(|| ext_php_rs::exception::PhpException::default("Opts must be an array".into()))?;
        
        let mut op = self.io.clone().broadcast();
        
        if let Some(rooms_zval) = opts_ht.get("rooms") {
             if let Some(rooms) = rooms_zval.array() {
                 for (_k, v) in rooms.iter() {
                     if let Some(r) = v.str() {
                         op = op.to(r.to_string());
                     }
                 }
             }
        }
        
        if let Some(except_zval) = opts_ht.get("except") {
             if let Some(except) = except_zval.array() {
                 for (_k, v) in except.iter() {
                    if let Some(id) = v.str() {
                         op = op.except(id.to_string());
                    }
                }
             }
        }

        let event = event.to_string();
        crate::runtime::context::spawn_local(async move {
            let _ = op.emit(event, &data).await;
        });

        Ok(())
    }
}
