use ext_php_rs::exception::PhpException;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::prelude::*;
use ext_php_rs::zend::{Function, FunctionEntry};
use std::ffi::CString;

// Define minimal FFI for Zend Function Table manipulation
// ext-php-rs doesn't expose EG(function_table) helpers publicly for modification easily.
mod zend {
    use std::ffi::{c_char, c_void};

    #[repr(C)]
    pub struct zend_string {
        pub gc: u64, // zend_refcounted_h
        pub h: u64,  // zend_ulong
        pub len: usize,
        pub val: [c_char; 1],
    }

    // Partial definition of zend_function
    #[repr(C)]
    pub struct zend_function {
        pub type_: u8,
        pub arg_flags: [u8; 3],
        pub fn_flags: u32,
        pub function_name: *mut zend_string,
        // ... other fields we hopefully don't need if we just swap pointers
        // actually, swapping the entire zend_function struct in the hash table is safer,
        // but we need to find the zval* in the hash table.
    }

    extern "C" {
        pub fn zend_hash_str_find(ht: *mut c_void, key: *const c_char, len: usize) -> *mut c_void;
        pub fn zend_hash_str_update(ht: *mut c_void, key: *const c_char, len: usize, data: *mut c_void) -> *mut c_void;
    }
}


// override_function moved to lib.rs


// Make the generated internal struct public so wrap_function! can see it in lib.rs
// The macro generates `_internal_override_function`.
// We can't easily change visibility of macro generated code.
// But wrap_function! usually works if the function is in the same crate.
// The issue "struct `crate::util::_internal_override_function` exists but is inaccessible"
// suggests the macro generated it as private in `util` module.
// We need to re-export it or move the function to lib.rs?
// Or maybe just `pub use` it? 
// Actually, wrap_function! expects the internal struct to be accessible. 
// Let's try `pub use`ing the internal struct if we knew its name, but it's generated.
// 
// WORKAROUND: Define this function in `lib.rs` or make `util` module inline?
// Or try to re-export the hidden struct (not possible by name easily).
//
// Let's move `override_function` to `lib.rs` for now to avoid this visibility issue 
// with cross-module macro usage in ext-php-rs.


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
