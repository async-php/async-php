use ext_php_rs::prelude::*;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::class::RegisteredClass;
use ext_php_rs::exception::PhpException;
use ext_php_rs::types::{ZendClassObject, Zval};
use tokio::io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite, BufReader};

use crate::{IO_BUF, IO_READ, IO_SEEK, IO_WRITE};
use crate::io::{
    AsyncBufReader, AsyncReadSeeker, AsyncReadWrite, AsyncReadWriteSeek, AsyncReadWriteSeeker,
    AsyncReadWriter, AsyncReadSeek, AsyncSeeker, AsyncWriteSeeker, AsyncWriteSeek, AsyncWriter,
    AsyncReader,
};
use crate::util::Shared;

pub const IO_MASK: i64 = IO_READ | IO_WRITE | IO_SEEK | IO_BUF;

const IO_READ_WRITE: i64 = IO_READ | IO_WRITE;
const IO_READ_SEEK: i64 = IO_READ | IO_SEEK;
const IO_WRITE_SEEK: i64 = IO_WRITE | IO_SEEK;
const IO_READ_WRITE_SEEK: i64 = IO_READ | IO_WRITE | IO_SEEK;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IoCastTarget {
    Reader,
    Writer,
    Seeker,
    BufReader,
    ReadWriter,
    ReadSeeker,
    WriteSeeker,
    ReadWriteSeeker,
}

fn parse_io_cast_target(flags: i64) -> PhpResult<IoCastTarget> {
    if flags == 0 {
        return Err(PhpException::default("IO cast type cannot be 0".into()));
    }

    if (flags & !IO_MASK) != 0 {
        return Err(PhpException::default(format!(
            "Invalid IO cast type bits: {}",
            flags
        )));
    }

    let wants_buf = (flags & IO_BUF) != 0;
    if wants_buf {
        let non_buf_bits = flags & !IO_BUF;
        if non_buf_bits == 0 || non_buf_bits == IO_READ {
            return Ok(IoCastTarget::BufReader);
        }
        return Err(PhpException::default(
            "BUF cannot be combined with WRITE/SEEK".into(),
        ));
    }

    match flags {
        IO_READ => Ok(IoCastTarget::Reader),
        IO_WRITE => Ok(IoCastTarget::Writer),
        IO_SEEK => Ok(IoCastTarget::Seeker),
        IO_READ_WRITE => Ok(IoCastTarget::ReadWriter),
        IO_READ_SEEK => Ok(IoCastTarget::ReadSeeker),
        IO_WRITE_SEEK => Ok(IoCastTarget::WriteSeeker),
        IO_READ_WRITE_SEEK => Ok(IoCastTarget::ReadWriteSeeker),
        _ => Err(PhpException::default(format!(
            "Unsupported IO cast type: {}",
            flags
        ))),
    }
}

pub trait AsyncIO {
    fn as_reader(&self) -> PhpResult<AsyncReader> {
        Err(PhpException::default("Not implemented".to_string()))
    }

    fn as_writer(&self) -> PhpResult<AsyncWriter> {
        Err(PhpException::default("Not implemented".to_string()))
    }

    fn as_seeker(&self) -> PhpResult<AsyncSeeker> {
        Err(PhpException::default("Not implemented".to_string()))
    }

    fn as_buf_reader(&self) -> PhpResult<AsyncBufReader> {
        Err(PhpException::default("Not implemented".to_string()))
    }

    fn as_read_writer(&self) -> PhpResult<AsyncReadWriter> {
        Err(PhpException::default("Not implemented".to_string()))
    }

    fn as_read_seeker(&self) -> PhpResult<AsyncReadSeeker> {
        Err(PhpException::default("Not implemented".to_string()))
    }

    fn as_write_seeker(&self) -> PhpResult<AsyncWriteSeeker> {
        Err(PhpException::default("Not implemented".to_string()))
    }

    fn as_read_write_seeker(&self) -> PhpResult<AsyncReadWriteSeeker> {
        Err(PhpException::default("Not implemented".to_string()))
    }
}

fn object_to_zval<T: RegisteredClass>(obj: T) -> PhpResult<Zval> {
    ZendClassObject::new(obj)
        .into_zval(false)
        .map_err(|e| PhpException::default(format!("Failed to convert object to Zval: {:?}", e)))
}

pub fn cast_io(io: &dyn AsyncIO, ty: i64) -> PhpResult<Zval> {
    let target = parse_io_cast_target(ty)?;

    match target {
        IoCastTarget::Reader => object_to_zval(io.as_reader()?),
        IoCastTarget::Writer => object_to_zval(io.as_writer()?),
        IoCastTarget::Seeker => object_to_zval(io.as_seeker()?),
        IoCastTarget::BufReader => object_to_zval(io.as_buf_reader()?),
        IoCastTarget::ReadWriter => object_to_zval(io.as_read_writer()?),
        IoCastTarget::ReadSeeker => object_to_zval(io.as_read_seeker()?),
        IoCastTarget::WriteSeeker => object_to_zval(io.as_write_seeker()?),
        IoCastTarget::ReadWriteSeeker => object_to_zval(io.as_read_write_seeker()?),
    }
}

impl AsyncIO for Shared<Box<dyn AsyncRead + Unpin + Send>> {
    fn as_reader(&self) -> PhpResult<AsyncReader> {
        Ok(AsyncReader::from_shared(self.clone()))
    }

    fn as_buf_reader(&self) -> PhpResult<AsyncBufReader> {
        Ok(AsyncBufReader::new(BufReader::new(self.clone())))
    }
}

impl AsyncIO for Shared<Box<dyn AsyncWrite + Unpin + Send>> {
    fn as_writer(&self) -> PhpResult<AsyncWriter> {
        Ok(AsyncWriter::from_shared(self.clone()))
    }
}

impl AsyncIO for Shared<Box<dyn AsyncSeek + Unpin + Send>> {
    fn as_seeker(&self) -> PhpResult<AsyncSeeker> {
        Ok(AsyncSeeker::from_shared(self.clone()))
    }
}

impl AsyncIO for Shared<Box<dyn AsyncBufRead + Unpin + Send>> {
    fn as_buf_reader(&self) -> PhpResult<AsyncBufReader> {
        Ok(AsyncBufReader::from_shared(self.clone()))
    }

    fn as_reader(&self) -> PhpResult<AsyncReader> {
        let trait_object: Shared<Box<dyn AsyncRead + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncReader::from_shared(trait_object))
    }
}

impl AsyncIO for Shared<Box<dyn AsyncReadWrite>> {
    fn as_read_writer(&self) -> PhpResult<AsyncReadWriter> {
        Ok(AsyncReadWriter::from_shared(self.clone()))
    }

    fn as_reader(&self) -> PhpResult<AsyncReader> {
        let trait_object: Shared<Box<dyn AsyncRead + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncReader::from_shared(trait_object))
    }

    fn as_writer(&self) -> PhpResult<AsyncWriter> {
        let trait_object: Shared<Box<dyn AsyncWrite + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncWriter::from_shared(trait_object))
    }

    fn as_buf_reader(&self) -> PhpResult<AsyncBufReader> {
        Ok(AsyncBufReader::new(BufReader::new(self.clone())))
    }
}

impl AsyncIO for Shared<Box<dyn AsyncReadSeek>> {
    fn as_read_seeker(&self) -> PhpResult<AsyncReadSeeker> {
        Ok(AsyncReadSeeker::from_shared(self.clone()))
    }

    fn as_reader(&self) -> PhpResult<AsyncReader> {
        let trait_object: Shared<Box<dyn AsyncRead + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncReader::from_shared(trait_object))
    }

    fn as_seeker(&self) -> PhpResult<AsyncSeeker> {
        let trait_object: Shared<Box<dyn AsyncSeek + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncSeeker::from_shared(trait_object))
    }

    fn as_buf_reader(&self) -> PhpResult<AsyncBufReader> {
        Ok(AsyncBufReader::new(BufReader::new(self.clone())))
    }
}

impl AsyncIO for Shared<Box<dyn AsyncWriteSeek>> {
    fn as_write_seeker(&self) -> PhpResult<AsyncWriteSeeker> {
        Ok(AsyncWriteSeeker::from_shared(self.clone()))
    }

    fn as_writer(&self) -> PhpResult<AsyncWriter> {
        let trait_object: Shared<Box<dyn AsyncWrite + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncWriter::from_shared(trait_object))
    }

    fn as_seeker(&self) -> PhpResult<AsyncSeeker> {
        let trait_object: Shared<Box<dyn AsyncSeek + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncSeeker::from_shared(trait_object))
    }
}

impl AsyncIO for Shared<Box<dyn AsyncReadWriteSeek>> {
    fn as_read_write_seeker(&self) -> PhpResult<AsyncReadWriteSeeker> {
        Ok(AsyncReadWriteSeeker::from_shared(self.clone()))
    }

    fn as_read_writer(&self) -> PhpResult<AsyncReadWriter> {
        let rw = AsyncReadWriter::new(self.clone());
        Ok(rw)
    }

    fn as_read_seeker(&self) -> PhpResult<AsyncReadSeeker> {
        let rs = AsyncReadSeeker::new(self.clone());
        Ok(rs)
    }

    fn as_write_seeker(&self) -> PhpResult<AsyncWriteSeeker> {
        let ws = AsyncWriteSeeker::new(self.clone());
        Ok(ws)
    }

    fn as_reader(&self) -> PhpResult<AsyncReader> {
        let trait_object: Shared<Box<dyn AsyncRead + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncReader::from_shared(trait_object))
    }

    fn as_writer(&self) -> PhpResult<AsyncWriter> {
        let trait_object: Shared<Box<dyn AsyncWrite + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncWriter::from_shared(trait_object))
    }

    fn as_seeker(&self) -> PhpResult<AsyncSeeker> {
        let trait_object: Shared<Box<dyn AsyncSeek + Unpin + Send>> = Shared::new(Box::new(self.clone()));
        Ok(AsyncSeeker::from_shared(trait_object))
    }

    fn as_buf_reader(&self) -> PhpResult<AsyncBufReader> {
        Ok(AsyncBufReader::new(BufReader::new(self.clone())))
    }
}
