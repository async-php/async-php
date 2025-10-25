use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt, SeekFrom};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;

/// Stateless filesystem operations (replacing file_*, is_*, etc.)
#[php_class]
#[php(name = "Async\\Driver\\Filesystem")]
pub struct AsyncFilesystem;

#[php_impl]
impl AsyncFilesystem {
    // --- Content Operations ---

    pub fn get_contents(path: String) -> RustFuture {
        let future = async move {
            match fs::read_to_string(PathBuf::from(path)).await {
                Ok(content) => {
                     let mut z = Zval::new();
                     z.set_string(&content, false).unwrap();
                     z
                }
                Err(_) => Zval::new(), // False/Null
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

    // --- Path Operations ---

    pub fn exists(path: String) -> RustFuture {
        let future = async move {
            let mut z = Zval::new();
            z.set_bool(fs::try_exists(path).await.unwrap_or(false));
            z
        };
        RustFuture::new(future)
    }

    pub fn is_file(path: String) -> RustFuture {
        let future = async move {
            let mut z = Zval::new();
            match fs::metadata(path).await {
                Ok(m) => z.set_bool(m.is_file()),
                Err(_) => z.set_bool(false),
            }
            z
        };
        RustFuture::new(future)
    }

    pub fn is_dir(path: String) -> RustFuture {
        let future = async move {
            let mut z = Zval::new();
            match fs::metadata(path).await {
                Ok(m) => z.set_bool(m.is_dir()),
                Err(_) => z.set_bool(false),
            }
            z
        };
        RustFuture::new(future)
    }

    pub fn unlink(path: String) -> RustFuture {
        let future = async move {
            let mut z = Zval::new();
            z.set_bool(fs::remove_file(path).await.is_ok());
            z
        };
        RustFuture::new(future)
    }

    pub fn rename(from: String, to: String) -> RustFuture {
        let future = async move {
            let mut z = Zval::new();
            z.set_bool(fs::rename(from, to).await.is_ok());
            z
        };
        RustFuture::new(future)
    }

    pub fn copy(from: String, to: String) -> RustFuture {
        let future = async move {
            let mut z = Zval::new();
            z.set_bool(fs::copy(from, to).await.is_ok());
            z
        };
        RustFuture::new(future)
    }

    pub fn mkdir(path: String) -> RustFuture {
        let future = async move {
            let mut z = Zval::new();
            z.set_bool(fs::create_dir_all(path).await.is_ok());
            z
        };
        RustFuture::new(future)
    }

    pub fn rmdir(path: String) -> RustFuture {
        let future = async move {
            let mut z = Zval::new();
            z.set_bool(fs::remove_dir(path).await.is_ok());
            z
        };
        RustFuture::new(future)
    }
    
    pub fn size(path: String) -> RustFuture {
        let future = async move {
            let mut z = Zval::new();
            match fs::metadata(path).await {
                Ok(m) => z.set_long(m.len() as i64),
                Err(_) => z.set_bool(false),
            }
            z
        };
        RustFuture::new(future)
    }
}

/// Stateful file handle (replacing fopen, fread, fwrite)
#[php_class]
#[php(name = "Async\\Driver\\FileHandle")]
pub struct AsyncFileHandle {
    // Rc<RefCell> allows single-threaded shared mutability without locking overhead.
    inner: Rc<RefCell<fs::File>>,
}

#[php_impl]
impl AsyncFileHandle {
    pub fn open(path: String, mode: String) -> RustFuture {
        let future = async move {
            let mut opts = fs::OpenOptions::new();
            
            match mode.as_str() {
                "r" | "rb" => { opts.read(true); },
                "w" | "wb" => { opts.write(true).create(true).truncate(true); },
                "a" | "ab" => { opts.append(true).create(true); },
                "r+" | "r+b" => { opts.read(true).write(true); },
                "w+" | "w+b" => { opts.read(true).write(true).create(true).truncate(true); },
                _ => { opts.read(true); } 
            };

            match opts.open(path).await {
                Ok(file) => {
                    let obj = AsyncFileHandle { inner: Rc::new(RefCell::new(file)) };
                    ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                }
                Err(_) => Zval::new(),
            }
        };
        RustFuture::new(future)
    }

    pub fn read(&self, length: usize) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
            let mut buf = vec![0u8; length];
            
            // Try to borrow mutably. If strictly used in one Fiber at a time, this works.
            // If user calls read() in two fibers on the same handle concurrently, this returns error immediately.
            if let Ok(mut lock) = file.try_borrow_mut() {
                match lock.read(&mut buf).await {
                    Ok(0) => {
                         let mut z = Zval::new();
                         z.set_string("", false).unwrap();
                         z
                    },
                    Ok(n) => {
                        buf.truncate(n);
                        let s = String::from_utf8_lossy(&buf).to_string();
                        let mut z = Zval::new();
                        z.set_string(&s, false).unwrap();
                        z
                    }
                    Err(_) => {
                         let mut z = Zval::new();
                         z.set_bool(false);
                         z
                    }
                }
            } else {
                // Resource Busy
                let mut z = Zval::new();
                z.set_bool(false); 
                z
            }
        };
        RustFuture::new(future)
    }

    pub fn write(&self, data: String) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
            if let Ok(mut lock) = file.try_borrow_mut() {
                match lock.write_all(data.as_bytes()).await {
                    Ok(_) => {
                        let mut z = Zval::new();
                        z.set_long(data.len() as i64);
                        z
                    }
                    Err(_) => {
                         let mut z = Zval::new();
                         z.set_bool(false);
                         z
                    }
                }
            } else {
                let mut z = Zval::new();
                z.set_bool(false); 
                z
            }
        };
        RustFuture::new(future)
    }
    
    pub fn seek(&self, pos: i64) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
            let mut z = Zval::new();
            if let Ok(mut lock) = file.try_borrow_mut() {
                match lock.seek(SeekFrom::Start(pos as u64)).await {
                    Ok(new_pos) => z.set_long(new_pos as i64),
                    Err(_) => z.set_bool(false),
                }
            } else {
                z.set_bool(false);
            }
            z
        };
        RustFuture::new(future)
    }

    pub fn close(&self) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
             let mut z = Zval::new();
             if let Ok(mut lock) = file.try_borrow_mut() {
                 let _ = lock.shutdown().await;
                 z.set_bool(true);
             } else {
                 z.set_bool(false);
             }
             z
        };
        RustFuture::new(future)
    }
}
