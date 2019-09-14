use std::fs::File;
use std::io::{self, IoSlice, IoSliceMut, Read, Write};
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle};
use std::ptr;

use mio::windows::SourceHandle; // FIXME: doesn't exists.
use mio::{event, Interests, Registry, Token};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::namedpipeapi::CreatePipe;

#[derive(Debug)]
pub struct Sender {
    inner: File,
}

impl event::Source for Sender {
    fn register(&self, registry: &Registry, token: Token, interests: Interests) -> io::Result<()> {
        SourceHandle(&self.inner.as_raw_handle()).register(registry, token, interests)
    }

    fn reregister(
        &self,
        registry: &Registry,
        token: Token,
        interests: Interests,
    ) -> io::Result<()> {
        SourceHandle(&self.inner.as_raw_handle()).reregister(registry, token, interests)
    }

    fn deregister(&self, registry: &Registry) -> io::Result<()> {
        SourceHandle(&self.inner.as_raw_handle()).deregister(registry)
    }
}

impl Write for Sender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        self.inner.write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl AsRawHandle for Sender {
    fn as_raw_handle(&self) -> RawHandle {
        self.inner.as_raw_handle()
    }
}

impl IntoRawHandle for Sender {
    fn into_raw_handle(self) -> RawHandle {
        self.inner.into_raw_handle()
    }
}

#[derive(Debug)]
pub struct Receiver {
    inner: File,
}

impl event::Source for Receiver {
    fn register(&self, registry: &Registry, token: Token, interests: Interests) -> io::Result<()> {
        SourceHandle(&self.inner.as_raw_handle()).register(registry, token, interests)
    }

    fn reregister(
        &self,
        registry: &Registry,
        token: Token,
        interests: Interests,
    ) -> io::Result<()> {
        SourceHandle(&self.inner.as_raw_handle()).reregister(registry, token, interests)
    }

    fn deregister(&self, registry: &Registry) -> io::Result<()> {
        SourceHandle(&self.inner.as_raw_handle()).deregister(registry)
    }
}

impl Read for Receiver {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.inner.read_vectored(bufs)
    }
}

impl AsRawHandle for Receiver {
    fn as_raw_handle(&self) -> RawHandle {
        self.inner.as_raw_handle()
    }
}

impl IntoRawHandle for Receiver {
    fn into_raw_handle(self) -> RawHandle {
        self.inner.into_raw_handle()
    }
}

pub fn new_pipe() -> io::Result<(Sender, Receiver)> {
    let mut r = INVALID_HANDLE_VALUE;
    let mut w = INVALID_HANDLE_VALUE;

    if CreatePipe(&mut r, &mut w, ptr::null_mut(), 0) == 0 {
        return Err(io::Error::last_os_error());
    }

    // FIXME: set non-blocking.

    let r = Receiver {
        inner: unsafe { File::from_raw_handle(r) },
    };
    let w = Sender {
        inner: unsafe { File::from_raw_handle(w) },
    };
    Ok((w, r))
}
