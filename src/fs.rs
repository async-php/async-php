use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt, SeekFrom};
use std::path::PathBuf;
use crate::util::Shared;

/// Stateless filesystem operations (replacing file_*, is_*, etc.)
#[php_class]
#[php(name = "Async\\Kernel\\FileSystem")]
pub struct AsyncFilesystem;

#[php_impl]
impl AsyncFilesystem {
    // --- Content Operations ---

    pub fn get_contents(path: String) -> RustFuture {
        let future = async move {
            fs::read_to_string(PathBuf::from(path)).await
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

    pub fn scandir(path: String) -> RustFuture {
        let future = async move {
            let mut entries = Vec::new();
            // Emulate PHP scandir behavior: include . and ..
            entries.push(".".to_string());
            entries.push("..".to_string());

            match fs::read_dir(path).await {
                Ok(mut dir) => {
                    while let Ok(Some(entry)) = dir.next_entry().await {
                         if let Ok(name) = entry.file_name().into_string() {
                             entries.push(name);
                         }
                    }
                    
                    let mut arr = ext_php_rs::types::ZendHashTable::new();
                    for name in entries {
                        let _ = arr.push(name);
                    }
                    arr.into_zval(false).unwrap_or_else(|_| Zval::new())
                }
                Err(_) => Zval::new() // False
            }
        };
        RustFuture::new(future)
    }
}

/// Stateful file handle (replacing fopen, fread, fwrite)
#[php_class]
#[php(name = "Async\\Kernel\\FileSystem\\FileHandle")]
pub struct AsyncFileHandle {
    // Shared wrapper allows single-threaded shared mutability without locking overhead.
    inner: Shared<fs::File>,
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
                    let obj = AsyncFileHandle { inner: Shared::new(file) };
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

            match file.get_mut().read(&mut buf).await {
                Ok(0) => {
                     let mut z = Zval::new();
                     z.set_string("", false).unwrap();
                     z
                },
                Ok(n) => {
                    buf.truncate(n);
                    let mut z = Zval::new();
                    z.set_binary(buf);
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

    pub fn write(&self, data: String) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
            match file.get_mut().write_all(data.as_bytes()).await {
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
        };
        RustFuture::new(future)
    }

    /// Flush the file, ensuring all buffered data is written to the OS
    pub fn flush(&self) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
            use tokio::io::AsyncWriteExt;
            let mut z = Zval::new();
            z.set_bool(file.get_mut().flush().await.is_ok());
            z
        };
        RustFuture::new(future)
    }

    /// Sync all data and metadata to disk (like fsync)
    pub fn sync_all(&self) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
            let mut z = Zval::new();
            z.set_bool(file.get_mut().sync_all().await.is_ok());
            z
        };
        RustFuture::new(future)
    }

    /// Sync only data to disk, not metadata (like fdatasync)
    pub fn sync_data(&self) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
            let mut z = Zval::new();
            z.set_bool(file.get_mut().sync_data().await.is_ok());
            z
        };
        RustFuture::new(future)
    }

    pub fn seek(&self, offset: i64, whence: i64) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
            let mut z = Zval::new();
            // Map whence parameter to SeekFrom
            // 0 = SEEK_START, 1 = SEEK_CURRENT, 2 = SEEK_END
            let seek_from = match whence {
                0 => SeekFrom::Start(offset as u64),
                1 => SeekFrom::Current(offset),
                2 => SeekFrom::End(offset),
                _ => SeekFrom::Start(offset as u64), // Default to SEEK_START
            };

            match file.get_mut().seek(seek_from).await {
                Ok(new_pos) => z.set_long(new_pos as i64),
                Err(_) => z.set_bool(false),
            }
            z
        };
        RustFuture::new(future)
    }

    pub fn close(&self) -> RustFuture {
        let file = self.inner.clone();
        let future = async move {
             let _ = file.get_mut().shutdown().await;
             let mut z = Zval::new();
             z.set_bool(true);
             z
        };
        RustFuture::new(future)
    }

    /// Extract as AsyncReader (returns \Async\Kernel\IO\AsyncReader)
    ///
    /// This allows using a FileHandle anywhere an AsyncReader is accepted,
    /// enabling zero-overhead streaming without creating a new fiber.
    #[php]
    pub fn as_reader(&self) -> crate::io::AsyncReader {
        use crate::io::AsyncReader;
        let trait_object: Shared<Box<dyn tokio::io::AsyncRead + Unpin + Send>> =
            Shared::new(Box::new(self.inner.clone()));
        AsyncReader::from_shared(trait_object)
    }

    /// Extract as AsyncWriter (returns \Async\Kernel\IO\AsyncWriter)
    #[php]
    pub fn as_writer(&self) -> crate::io::AsyncWriter {
        use crate::io::AsyncWriter;
        let trait_object: Shared<Box<dyn tokio::io::AsyncWrite + Unpin + Send>> =
            Shared::new(Box::new(self.inner.clone()));
        AsyncWriter::from_shared(trait_object)
    }

    /// Extract as AsyncSeeker (returns \Async\Kernel\IO\AsyncSeeker)
    #[php]
    pub fn as_seeker(&self) -> crate::io::AsyncSeeker {
        use crate::io::AsyncSeeker;
        let trait_object: Shared<Box<dyn tokio::io::AsyncSeek + Unpin + Send>> =
            Shared::new(Box::new(self.inner.clone()));
        AsyncSeeker::from_shared(trait_object)
    }

    /// Extract as AsyncReadWriter (returns \Async\Kernel\IO\AsyncReadWriter)
    #[php]
    pub fn as_read_writer(&self) -> crate::io::AsyncReadWriter {
        use crate::io::AsyncReadWriter;
        let trait_object: Shared<Box<dyn crate::io::AsyncReadWrite>> =
            Shared::new(Box::new(self.inner.clone()));
        AsyncReadWriter::from_shared(trait_object)
    }
}

impl AsyncFileHandle {
    /// Internal: Get inner Shared<File> for zero-copy operations
    pub(crate) fn get_inner(&self) -> Shared<fs::File> {
        self.inner.clone()
    }
}
