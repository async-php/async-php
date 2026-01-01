use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use socketioxide::SocketIo;
use socketioxide::extract::{SocketRef, Data};
use socketioxide::layer::SocketIoLayer;
use socketioxide::socket::DisconnectReason;
use crate::util::{zval_to_json, json_to_zval, Shared, UnsafeZval};
use crate::runtime::runtime::{call_closure_async, call_method_async};

#[php_class]
#[php(name = "Async\\Kernel\\Network\\Http\\SocketIo\\Socket")]
pub struct AsyncSocket {
    inner: SocketRef,
    adapter: Option<Shared<UnsafeZval>>,
}

#[php_impl]
impl AsyncSocket {
    #[php]
    pub fn id(&self) -> String {
        self.inner.id.to_string()
    }

    #[php]
    pub fn emit(&self, event: String, data: Option<&Zval>) -> PhpResult<()> {
        let val = if let Some(d) = data { zval_to_json(d) } else { serde_json::Value::Null };
        self.inner.emit(event, &val).map_err(|e| {
            ext_php_rs::exception::PhpException::default(format!("Emit error: {}", e))
        })?;
        Ok(())
    }
    
    #[php]
    pub fn join(&self, room: String) -> PhpResult<()> {
        self.inner.join(room.clone());
        if let Some(adapter) = &self.adapter {
             let adapter = adapter.clone();
             let id = self.inner.id.to_string();
             crate::runtime::context::spawn_local(async move {
                 let id_zval = id.into_zval(false).unwrap_or_default();
                 let room_zval = room.into_zval(false).unwrap_or_default();
                 let _ = call_method_async(adapter.0.shallow_clone(), "add", vec![id_zval, room_zval]).await;
             });
        }
        Ok(())
    }

    #[php]
    pub fn leave(&self, room: String) -> PhpResult<()> {
        self.inner.leave(room.clone());
        if let Some(adapter) = &self.adapter {
             let adapter = adapter.clone();
             let id = self.inner.id.to_string();
             crate::runtime::context::spawn_local(async move {
                 let id_zval = id.into_zval(false).unwrap_or_default();
                 let room_zval = room.into_zval(false).unwrap_or_default();
                 let _ = call_method_async(adapter.0.shallow_clone(), "del", vec![id_zval, room_zval]).await;
             });
        }
        Ok(())
    }

    #[php]
    pub fn broadcast(&self, event: String, data: Option<&Zval>) -> PhpResult<()> {
        let val = if let Some(d) = data { zval_to_json(d) } else { serde_json::Value::Null };
        let socket = self.inner.clone();
        let event_clone = event.clone();
        
        crate::runtime::context::spawn_local(async move {
            let _ = socket.broadcast().emit(event_clone, &val).await;
        });

        if let Some(adapter) = &self.adapter {
             let adapter = adapter.clone();
             let id = self.inner.id.to_string();
             let mut packet = ext_php_rs::types::ZendHashTable::new();
             let _ = packet.push(event.into_zval(false).unwrap_or_default());
             if let Some(d) = data { let _ = packet.push(d.shallow_clone()); }
             let packet_zval = packet.into_zval(false).unwrap_or_default();
             
             let mut opts = ext_php_rs::types::ZendHashTable::new();
             let mut except = ext_php_rs::types::ZendHashTable::new();
             let _ = except.push(id.into_zval(false).unwrap_or_default());
             let _ = opts.insert("except", except.into_zval(false).unwrap_or_default());
             let opts_zval = opts.into_zval(false).unwrap_or_default();
             
             crate::runtime::context::spawn_local(async move {
                 let _ = call_method_async(adapter.0.shallow_clone(), "broadcast", vec![packet_zval, opts_zval]).await;
             });
        }
        Ok(())
    }

    #[php]
    pub fn to(&self, room: String, event: String, data: Option<&Zval>) -> PhpResult<()> {
        let val = if let Some(d) = data { zval_to_json(d) } else { serde_json::Value::Null };
        let socket = self.inner.clone();
        let room_clone = room.clone();
        let event_clone = event.clone();
        
        crate::runtime::context::spawn_local(async move {
            let _ = socket.to(room_clone).emit(event_clone, &val).await;
        });
        
        if let Some(adapter) = &self.adapter {
             let adapter = adapter.clone();
             let id = self.inner.id.to_string();
             let mut packet = ext_php_rs::types::ZendHashTable::new();
             let _ = packet.push(event.into_zval(false).unwrap_or_default());
             if let Some(d) = data { let _ = packet.push(d.shallow_clone()); }
             let packet_zval = packet.into_zval(false).unwrap_or_default();
             
             let mut opts = ext_php_rs::types::ZendHashTable::new();
             let mut rooms_ht = ext_php_rs::types::ZendHashTable::new();
             let _ = rooms_ht.push(room.into_zval(false).unwrap_or_default());
             let _ = opts.insert("rooms", rooms_ht.into_zval(false).unwrap_or_default());
             let mut except = ext_php_rs::types::ZendHashTable::new();
             let _ = except.push(id.into_zval(false).unwrap_or_default());
             let _ = opts.insert("except", except.into_zval(false).unwrap_or_default());
             let opts_zval = opts.into_zval(false).unwrap_or_default();
             
             crate::runtime::context::spawn_local(async move {
                 let _ = call_method_async(adapter.0.shallow_clone(), "broadcast", vec![packet_zval, opts_zval]).await;
             });
        }
        Ok(())
    }

    #[php]
    pub fn on(&self, event: String, callback: &Zval) -> PhpResult<()> {
        let cb = Shared::new(UnsafeZval(callback.shallow_clone()));
        let adapter = self.adapter.clone();
        self.inner.on(event, move |socket: SocketRef, data: Data<serde_json::Value>| {
            let cb = cb.clone();
            let adapter = adapter.clone();
            async move {
                crate::runtime::context::spawn_local(async move {
                    let socket_obj = AsyncSocket { inner: socket, adapter };
                    let socket_zval = ext_php_rs::types::ZendClassObject::new(socket_obj).into_zval(false).unwrap_or_default();
                    let data_zval = json_to_zval(&data.0);
                    let _ = call_closure_async(cb.0.shallow_clone(), vec![data_zval, socket_zval]).await;
                });
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
    adapter: Option<Shared<UnsafeZval>>,
}

#[php_impl]
impl AsyncSocketIo {
    #[php]
    pub fn __construct(adapter: Option<&Zval>) -> Self {
        let (layer, io) = SocketIo::new_layer();
        Self {
            layer: Some(layer),
            io,
            adapter: adapter.map(|z| Shared::new(UnsafeZval(z.shallow_clone()))),
        }
    }

    #[php]
    pub fn on_connection(&mut self, namespace: String, callback: &Zval) -> PhpResult<()> {
        let cb = Shared::new(UnsafeZval(callback.shallow_clone()));
        let adapter = self.adapter.clone();
        self.io.ns(namespace, move |socket: SocketRef| {
            let cb = cb.clone();
            let adapter = adapter.clone();
            async move {
                crate::runtime::context::spawn_local(async move {
                    let socket_obj = AsyncSocket { inner: socket.clone(), adapter: adapter.clone() };
                    let socket_zval = ext_php_rs::types::ZendClassObject::new(socket_obj).into_zval(false).unwrap_or_default();
                    let _ = call_closure_async(cb.0.shallow_clone(), vec![socket_zval]).await;
                    
                    if let Some(a) = adapter {
                        let id = socket.id.to_string();
                        socket.on_disconnect(move |_: DisconnectReason| {
                            let a = a.clone();
                            let id = id.clone();
                            async move {
                                crate::runtime::context::spawn_local(async move {
                                    let id_zval = id.into_zval(false).unwrap_or_default();
                                    let _ = call_method_async(a.0.shallow_clone(), "delAll", vec![id_zval]).await;
                                });
                            }
                        });
                    }
                });
            }
        });
        Ok(())
    }

    #[php]
    pub fn publish(&self, packet: &Zval, opts: &Zval) -> PhpResult<()> {
        let packet_ht = packet.array().ok_or_else(|| PhpException::default("Packet must be array".into()))?;
        let event = packet_ht.get_index(0).and_then(|z| z.string()).unwrap_or_default();
        let data = packet_ht.get_index(1).map(|z| zval_to_json(z)).unwrap_or(serde_json::Value::Null);
        let opts_ht = opts.array().ok_or_else(|| PhpException::default("Opts must be array".into()))?;
        
        let mut op = self.io.clone().broadcast();
        if let Some(rooms) = opts_ht.get("rooms").and_then(|z| z.array()) {
            for (_, v) in rooms.iter() { if let Some(r) = v.string() { op = op.to(r); } }
        }
        if let Some(except) = opts_ht.get("except").and_then(|z| z.array()) {
            for (_, v) in except.iter() { if let Some(id) = v.string() { op = op.except(id); } }
        }

        crate::runtime::context::spawn_local(async move {
            let _ = op.emit(event, &data).await;
        });
        Ok(())
    }
}