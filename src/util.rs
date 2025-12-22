use ext_php_rs::exception::PhpException;
use ext_php_rs::types::{Zval, ZendHashTable};
use ext_php_rs::convert::IntoZval;

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Helper to convert a Result into a Zval (on success) or throw an exception (on error).
#[allow(dead_code)]
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
            Zval::new() // Return null, but exception is pending.
        }
    }
}

/// Create a PHP tuple (indexed array) from two values.
///
/// This enables Go-style multiple return values in PHP via array destructuring:
///
/// # Example (PHP side)
/// ```php
/// [$value, $ok] = $channel->pop();
/// if ($ok) {
///     echo "Received: $value\n";
/// } else {
///     echo "Channel closed or timeout\n";
/// }
/// ```
///
/// # Example (Rust side)
/// ```rust
/// use crate::util::tuple2;
///
/// // Return [value, true] on success
/// tuple2(received_value, true)
///
/// // Return [null, false] on failure
/// tuple2(Zval::new(), false)
/// ```
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

/// Create a PHP tuple (indexed array) from three values.
///
/// Similar to `tuple2` but for three values.
///
/// # Example (PHP side)
/// ```php
/// [$value, $error, $ok] = $operation->execute();
/// ```
#[allow(dead_code)]
pub fn tuple3<T1, T2, T3>(val1: T1, val2: T2, val3: T3) -> Zval
where
    T1: IntoZval,
    T2: IntoZval,
    T3: IntoZval,
{
    let mut ht = ZendHashTable::new();
    ht.insert(0i64, val1).ok();
    ht.insert(1i64, val2).ok();
    ht.insert(2i64, val3).ok();
    ht.into_zval(false).unwrap_or_else(|_| Zval::new())
}

/// A shared mutable container that allows multiple clones to access
/// the same value.
///
/// This type hides all unsafe internals and provides a 100% safe API
/// externally. It is useful for FFI scenarios where Rust's borrow
/// checker cannot model the lifetime rules of the foreign language.
///
/// WARNING:
/// --------
/// This type does not enforce Rust's aliasing rules. Multiple clones
/// may obtain &mut T at the same time, which is undefined behavior in
/// pure Rust. It is safe ONLY if the user guarantees exclusive mutation.
///
/// In typical FFI scenarios (single-threaded, externally-coordinated),
/// this is acceptable.
pub struct Shared<T> {
    inner: Arc<UnsafeCell<T>>,
}

// Allow sending across threads if T is Send.
unsafe impl<T: Send> Send for Shared<T> {}

// Allow sharing between threads if T is Send + Sync.
// NOTE: Sync here does NOT guarantee thread safety of mutation.
//       It only states that the container itself can be shared.
//       Actual safety must be guaranteed by the user.
unsafe impl<T: Send + Sync> Sync for Shared<T> {}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Shared<T> {
    /// Create a new SharedMut containing the given value.
    pub fn new(value: T) -> Self {
        Shared {
            inner: Arc::new(UnsafeCell::new(value)),
        }
    }

    /// Get a mutable reference to the inner value.
    ///
    /// This is safe from Rust's point of view because the caller cannot
    /// observe the internal aliasing logic. However, it is the user's
    /// responsibility to ensure no aliasing violations occur.
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }

    /// Get an immutable reference to the inner value.
    pub fn get_ref(&self) -> &T {
        unsafe { &*self.inner.get() }
    }
}

/// Deref allows `*shared_mut` to give &T.
/// This enables typical access patterns like:
///     let x = *shared;
impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}

/// DerefMut allows `*shared_mut = ...` or passing `&mut T` to functions.
///
/// WARNING: This does not prevent multiple `&mut T` borrows from multiple
/// clones. The user must coordinate correctness manually.
impl<T> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.get() }
    }
}

pub fn zval_to_json(zval: &Zval) -> serde_json::Value {
    if zval.is_null() {
        return serde_json::Value::Null;
    }
    if let Some(b) = zval.bool() {
        return serde_json::Value::Bool(b);
    }
    if let Some(l) = zval.long() {
        return serde_json::Value::Number(l.into());
    }
    if let Some(d) = zval.double() {
        if let Some(n) = serde_json::Number::from_f64(d) {
             return serde_json::Value::Number(n);
        }
    }
    if let Some(s) = zval.string() {
        return serde_json::Value::String(s);
    }
    if let Some(arr) = zval.array() {
        if arr.is_empty() {
            return serde_json::Value::Array(vec![]);
        }
        
        let mut is_list = true;
        let mut expected_idx = 0;
        let mut values = Vec::new();
        let mut map = serde_json::Map::new();
        
        for (k, v) in arr.iter() {
            match k {
                 ext_php_rs::types::ArrayKey::Long(idx) => {
                     if idx != expected_idx as i64 {
                         is_list = false;
                     }
                     expected_idx += 1;
                     map.insert(idx.to_string(), zval_to_json(v));
                     values.push(zval_to_json(v));
                 }
                 ext_php_rs::types::ArrayKey::Str(s) => {
                     is_list = false;
                     map.insert(s.to_string(), zval_to_json(v));
                 }
                 ext_php_rs::types::ArrayKey::String(s) => {
                     is_list = false;
                     map.insert(s, zval_to_json(v));
                 }
            }
        }
        
        if is_list {
             return serde_json::Value::Array(values);
        } else {
             return serde_json::Value::Object(map);
        }
    }
    
    // Fallback for objects/etc
    serde_json::Value::String("unsupported type".to_string())
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
            if let Some(i) = n.as_i64() {
                z.set_long(i);
            } else if let Some(f) = n.as_f64() {
                z.set_double(f);
            }
            z
        }
        serde_json::Value::String(s) => {
            let mut z = Zval::new();
            z.set_string(s, false).ok();
            z
        }
        serde_json::Value::Array(arr) => {
            let mut ht = ZendHashTable::new();
            for val in arr {
                ht.push(json_to_zval(val)).ok();
            }
            ht.into_zval(false).unwrap_or_else(|_| Zval::new())
        }
        serde_json::Value::Object(obj) => {
            let mut ht = ZendHashTable::new();
            for (k, v) in obj {
                ht.insert(k.as_str(), json_to_zval(v)).ok();
            }
            ht.into_zval(false).unwrap_or_else(|_| Zval::new())
        }
    }
}
