use ext_php_rs::exception::PhpException;
use ext_php_rs::types::{Zval, ZendHashTable};
use ext_php_rs::convert::IntoZval;

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Helper to convert a Result into a Zval (on success) or throw an exception (on error).
pub fn result_to_zval<T, E>(result: Result<T, E>) -> Zval
where
    T: IntoZval,
    E: std::fmt::Display,
{
    match result {
        Ok(val) => val.into_zval(false).unwrap_or_else(|_| Zval::new()),
        Err(e) => {
            let msg = format!("{}", e);
            let _ = PhpException::default(msg).throw();
            Zval::new()
        }
    }
}

pub fn tuple2<T1, T2>(val1: T1, val2: T2) -> Zval
where
    T1: IntoZval,
    T2: IntoZval,
{
    let mut ht = ZendHashTable::new();
    ht.insert(0i64, val1).ok();
    ht.insert(1i64, val2).ok();
    ht.into_zval(false).unwrap_or_else(|_| Zval::new())
}

/// A shared mutable container.
pub struct Shared<T> {
    inner: Arc<UnsafeCell<T>>,
}

unsafe impl<T: Send> Send for Shared<T> {}
unsafe impl<T: Send + Sync> Sync for Shared<T> {}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Shared {
            inner: Arc::new(UnsafeCell::new(value)),
        }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }

    pub fn get_ref(&self) -> &T {
        unsafe { &*self.inner.get() }
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl<T> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

/// Wrapper for Zval to allow Send/Sync.
pub struct UnsafeZval(pub Zval);
unsafe impl Send for UnsafeZval {}
unsafe impl Sync for UnsafeZval {}

impl Clone for UnsafeZval {
    fn clone(&self) -> Self {
        Self(self.0.shallow_clone())
    }
}

pub fn zval_to_json(zval: &Zval) -> serde_json::Value {
    if zval.is_null() { return serde_json::Value::Null; }
    if let Some(b) = zval.bool() { return serde_json::Value::Bool(b); }
    if let Some(l) = zval.long() { return serde_json::Value::Number(l.into()); }
    if let Some(d) = zval.double() {
        if let Some(n) = serde_json::Number::from_f64(d) { return serde_json::Value::Number(n); }
    }
    if let Some(s) = zval.string() { return serde_json::Value::String(s); }
    if let Some(arr) = zval.array() {
        if arr.is_empty() { return serde_json::Value::Array(vec![]); }
        let mut is_list = true;
        let mut expected_idx = 0;
        let mut values = Vec::new();
        let mut map = serde_json::Map::new();
        for (k, v) in arr.iter() {
            match k {
                 ext_php_rs::types::ArrayKey::Long(idx) => {
                     if idx != expected_idx as i64 { is_list = false; }
                     expected_idx += 1;
                     map.insert(idx.to_string(), zval_to_json(v));
                     values.push(zval_to_json(v));
                 }
                 ext_php_rs::types::ArrayKey::Str(s) => { is_list = false; map.insert(s.to_string(), zval_to_json(v)); }
                 ext_php_rs::types::ArrayKey::String(s) => { is_list = false; map.insert(s, zval_to_json(v)); }
            }
        }
        if is_list { serde_json::Value::Array(values) } else { serde_json::Value::Object(map) }
    } else {
        serde_json::Value::String("unsupported".into())
    }
}

pub fn json_to_zval(json: &serde_json::Value) -> Zval {
    match json {
        serde_json::Value::Null => Zval::new(),
        serde_json::Value::Bool(b) => {
            let mut z = Zval::new();
            z.set_bool(*b);
            z
        }
        serde_json::Value::Number(n) => {
            let mut z = Zval::new();
            if let Some(i) = n.as_i64() { z.set_long(i); } else if let Some(f) = n.as_f64() { z.set_double(f); }
            z
        }
        serde_json::Value::String(s) => {
            let mut z = Zval::new();
            z.set_string(s, false).ok();
            z
        }
        serde_json::Value::Array(arr) => {
            let mut ht = ZendHashTable::new();
            for val in arr { ht.push(json_to_zval(val)).ok(); }
            ht.into_zval(false).unwrap_or_default()
        }
        serde_json::Value::Object(obj) => {
            let mut ht = ZendHashTable::new();
            for (k, v) in obj { ht.insert(k.as_str(), json_to_zval(v)).ok(); }
            ht.into_zval(false).unwrap_or_default()
        }
    }
}