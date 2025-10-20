use ext_php_rs::exception::PhpException;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;

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
            // We effectively throw an exception here. 
            // Note: In ext-php-rs, returning Err(PhpException) from a #[php_method] 
            // is the standard way. This helper is for logic inside async blocks 
            // where we can't easily "return" an exception to the engine directly 
            // without returning a Zval that indicates failure or throwing manually.
            //
            // Since we are inside a RustFuture which returns a Zval to Fiber::suspend,
            // we have two choices:
            // 1. Return a Zval representing an error object.
            // 2. Throw the exception via Zend API immediately (dangerous in async?).
            //
            // Strategy: Return a special wrapper or rely on the caller to check.
            // BETTER STRATEGY for this Architecture:
            // Our RustFuture returns a Zval. If that Zval is an Exception object,
            // PHP userland (Kernel.php) could throw it?
            // OR: We just throw it here using `PhpException::default(msg).throw()`.
            
            let msg = format!("{}", e);
            let _ = PhpException::default(msg).throw();
            Zval::new() // Return null, but exception is pending.
        }
    }
}
