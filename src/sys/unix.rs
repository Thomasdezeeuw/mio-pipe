use std::fs::File;
use std::io::{self, IoSlice, IoSliceMut, Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

use mio::unix::SourceFd;
use mio::{event, Interest, Registry, Token};

#[derive(Debug)]
pub struct Sender {
    inner: File,
}

impl Sender {
    pub(crate) fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        set_nonblocking(self.inner.as_raw_fd(), nonblocking)
    }
}

impl event::Source for Sender {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).deregister(registry)
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

impl FromRawFd for Sender {
    unsafe fn from_raw_fd(fd: RawFd) -> Sender {
        Sender {
            inner: File::from_raw_fd(fd),
        }
    }
}

impl AsRawFd for Sender {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl IntoRawFd for Sender {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_raw_fd()
    }
}

#[derive(Debug)]
pub struct Receiver {
    inner: File,
}

impl Receiver {
    pub(crate) fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        set_nonblocking(self.inner.as_raw_fd(), nonblocking)
    }
}

impl event::Source for Receiver {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        SourceFd(&self.inner.as_raw_fd()).deregister(registry)
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

impl FromRawFd for Receiver {
    unsafe fn from_raw_fd(fd: RawFd) -> Receiver {
        Receiver {
            inner: File::from_raw_fd(fd),
        }
    }
}

impl AsRawFd for Receiver {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl IntoRawFd for Receiver {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_raw_fd()
    }
}

fn set_nonblocking(fd: RawFd, nonblocking: bool) -> io::Result<()> {
    let value = nonblocking as libc::c_int;
    if unsafe { libc::ioctl(fd, libc::FIONBIO, &value) } == -1 {
        return Err(io::Error::last_os_error());
    } else {
        Ok(())
    }
}

pub fn new_pipe() -> io::Result<(Sender, Receiver)> {
    let mut fds: [RawFd; 2] = [-1, -1];

    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    unsafe {
        if libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) != 0 {
            return Err(io::Error::last_os_error());
        }
    }

    #[cfg(any(target_os = "ios", target_os = "macos", target_os = "solaris"))]
    unsafe {
        // For platforms that don't have `pipe2(2)` we need to manually set the
        // correct flags on the file descriptor.
        if libc::pipe(fds.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }

        for fd in &fds {
            if libc::fcntl(*fd, libc::F_SETFL, libc::O_NONBLOCK) != 0
                || libc::fcntl(*fd, libc::F_SETFD, libc::FD_CLOEXEC) != 0
            {
                let err = io::Error::last_os_error();
                // Don't leak file descriptors. Can't handle error though.
                let _ = libc::close(fds[0]);
                let _ = libc::close(fds[1]);
                return Err(err);
            }
        }
    }

    let r = Receiver {
        inner: unsafe { File::from_raw_fd(fds[0]) },
    };
    let w = Sender {
        inner: unsafe { File::from_raw_fd(fds[1]) },
    };
    Ok((w, r))
}
