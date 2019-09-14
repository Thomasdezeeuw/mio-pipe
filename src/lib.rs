//! Unix pipe for use with Mio.
//!
//! See the [`new_pipe`] documentation.

use std::io::{self, IoSlice, IoSliceMut, Read, Write};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

use mio::{event, Interests, Registry, Token};

mod sys;

/// Sending end of an Unix pipe.
///
/// See [`new_pipe`] for documentation, including examples.
#[derive(Debug)]
pub struct Sender {
    inner: sys::Sender,
}

impl event::Source for Sender {
    fn register(&self, registry: &Registry, token: Token, interests: Interests) -> io::Result<()> {
        self.inner.register(registry, token, interests)
    }

    fn reregister(
        &self,
        registry: &Registry,
        token: Token,
        interests: Interests,
    ) -> io::Result<()> {
        self.inner.reregister(registry, token, interests)
    }

    fn deregister(&self, registry: &Registry) -> io::Result<()> {
        self.inner.deregister(registry)
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

#[cfg(unix)]
impl AsRawFd for Sender {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(unix)]
impl IntoRawFd for Sender {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_raw_fd()
    }
}

/// Receiving end of an Unix pipe.
///
/// See [`new_pipe`] for documentation, including examples.
#[derive(Debug)]
pub struct Receiver {
    inner: sys::Receiver,
}

impl event::Source for Receiver {
    fn register(&self, registry: &Registry, token: Token, interests: Interests) -> io::Result<()> {
        self.inner.register(registry, token, interests)
    }

    fn reregister(
        &self,
        registry: &Registry,
        token: Token,
        interests: Interests,
    ) -> io::Result<()> {
        self.inner.reregister(registry, token, interests)
    }

    fn deregister(&self, registry: &Registry) -> io::Result<()> {
        self.inner.deregister(registry)
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

#[cfg(unix)]
impl AsRawFd for Receiver {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(unix)]
impl IntoRawFd for Receiver {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_raw_fd()
    }
}

/// Create a new non-blocking Unix pipe.
///
/// This is a wrapper around Unix's `pipe(2)` system call and can be used as
/// inter-process or thread communication channel.
///
/// This channel may be created before forking the process and then one end used
/// in each process, e.g. the parent process has the sending end to send command
/// to the child process.
///
/// # Deregistering
///
/// Both `Sender` and `Receiver` will deregister themselves when dropped,
/// **iff** the file descriptors are not duplicated (via `dup(2)`).
///
/// # Examples
///
/// ```
/// use std::io::{self, Read, Write};
///
/// use mio::{Poll, Events, Interests, Token};
/// use mio_pipe::new_pipe;
///
/// // Unique tokens for the two ends of the channel.
/// const CHANNEL_RECV: Token = Token(0);
/// const CHANNEL_SEND: Token = Token(1);
///
/// # fn main() -> io::Result<()> {
/// let mut poll = Poll::new()?;
/// let mut events = Events::with_capacity(8);
///
/// // Create a new pipe.
/// let (mut sender, mut receiver) = new_pipe()?;
///
/// // Register both ends of the channel.
/// poll.registry().register(&mut receiver, CHANNEL_RECV, Interests::READABLE)?;
/// poll.registry().register(&mut sender, CHANNEL_SEND, Interests::WRITABLE)?;
///
/// const MSG: &[u8; 11] = b"Hello world";
///
/// loop {
///     poll.poll(&mut events, None)?;
///
///     for event in events.iter() {
///         match event.token() {
///             CHANNEL_SEND => sender.write(MSG)
///                 .and_then(|n| if n != MSG.len() {
///                         // We'll consider a short write an error in this
///                         // example. NOTE: we can't use `write_all` with
///                         // non-blocking I/O.
///                         Err(io::ErrorKind::WriteZero.into())
///                     } else {
///                         Ok(())
///                     })?,
///             CHANNEL_RECV => {
///                 let mut buf = [0; 11];
///                 let n = receiver.read(&mut buf)?;
///                 println!("received: {:?}", &buf[0..n]);
///                 assert_eq!(n, MSG.len());
///                 assert_eq!(&buf, &*MSG);
///                 return Ok(());
///             },
///             _ => unreachable!(),
///         }
///     }
/// }
/// # }
/// ```
pub fn new_pipe() -> io::Result<(Sender, Receiver)> {
    sys::new_pipe()
        .map(|(sender, receiver)| (Sender { inner: sender }, Receiver { inner: receiver }))
}
