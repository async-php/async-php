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
            let msg = format!("{}", e);
            let _ = PhpException::default(msg).throw();
            Zval::new() // Return null, but exception is pending.
        }
    }
}
