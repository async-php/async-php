use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::future::RustFuture;
use tokio::fs;
use std::path::PathBuf;

#[php_class]
pub struct AsyncFile;

#[php_impl]
impl AsyncFile {
    pub fn get_contents(path: String) -> RustFuture {
        let future = async move {
            match fs::read_to_string(PathBuf::from(path)).await {
                Ok(content) => {
                     let mut z = Zval::new();
                     z.set_string(&content, false).unwrap();
                     z
                }
                Err(_) => {
                     let mut z = Zval::new();
                     z.set_bool(false);
                     z
                }
            }
        };
        RustFuture::new(future)
    }

    pub fn put_contents(path: String, contents: String) -> RustFuture {
        let future = async move {
            match fs::write(PathBuf::from(path), contents).await {
                Ok(_) => {
                     let mut z = Zval::new();
                     z.set_bool(true);
                     z
                }
                Err(_) => {
                     let mut z = Zval::new();
                     z.set_bool(false);
                     z
                }
            }
        };
        RustFuture::new(future)
    }
}
