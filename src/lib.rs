//! Unix pipe for use with Mio.
//!
//! See the [`new_pipe`] documentation.
//!
//! ## Supported platforms
//!
//! Currently supported platforms:
//!
//! * Android
//! * DragonFly BSD
//! * FreeBSD
//! * Linux
//! * NetBSD
//! * OpenBSD
//! * iOS
//! * macOS
//!
//! The most notable exception in the list is Windows. If you want to contribute
//! a port to Windows please see [issue #4].
//!
//! [issue #4]: https://github.com/Thomasdezeeuw/mio-pipe/issues/6

use std::io::{self, IoSlice, IoSliceMut, Read, Write};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

use mio::{event, Interest, Registry, Token};

mod sys;

/// Sending end of an Unix pipe.
///
/// See [`new_pipe`] for documentation, including examples.
#[derive(Debug)]
pub struct Sender {
    inner: sys::Sender,
}

impl event::Source for Sender {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        self.inner.register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        self.inner.reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
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
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        self.inner.register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        self.inner.reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
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
/// This is a wrapper around Unix's [`pipe(2)`] system call and can be used as
/// inter-process or thread communication channel.
///
/// This channel may be created before forking the process and then one end used
/// in each process, e.g. the parent process has the sending end to send command
/// to the child process.
///
/// [`pipe(2)`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/pipe.html
///
/// # Events
///
/// The [`Sender`] can be registered with [`WRITABLE`] interest to receive
/// [writable events], the [`Receiver`] with [`READABLE`] interest. Once data is
/// written to the `Sender` the `Receiver` will receive an [readable event].
///
/// In addition to those events, events will also be generated if the other side
/// is dropped. However due to platform differences checking `is_{read,
/// write}_closed` is not enough. To check if the `Sender` is dropped you'll
/// need to check both [`is_error`] and [`is_read_closed`] on events for the
/// `Receiver`, if either is true the `Sender` is dropped. On the `Sender` end
/// check `is_error` and [`is_write_closed`], if either is true the `Receiver`
/// was dropped. Also see the second example below.
///
/// [`WRITABLE`]: Interest::WRITABLE
/// [writable events]: mio::event::Event::is_writable
/// [`READABLE`]: Interest::READABLE
/// [readable event]: mio::event::Event::is_readable
/// [`is_error`]: mio::event::Event::is_error
/// [`is_read_closed`]: mio::event::Event::is_read_closed
/// [`is_write_closed`]: mio::event::Event::is_write_closed
///
/// # Deregistering
///
/// Both `Sender` and `Receiver` will deregister themselves when dropped,
/// **iff** the file descriptors are not duplicated (via [`dup(2)`]).
///
/// [`dup(2)`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/dup.html
///
/// # Examples
///
/// Simple example that writes data into the sending end and read it from the
/// receiving end.
///
/// ```
/// use std::io::{self, Read, Write};
///
/// use mio::{Poll, Events, Interest, Token};
/// use mio_pipe::new_pipe;
///
/// // Unique tokens for the two ends of the channel.
/// const PIPE_RECV: Token = Token(0);
/// const PIPE_SEND: Token = Token(1);
///
/// # fn main() -> io::Result<()> {
/// // Create our `Poll` instance and the `Events` container.
/// let mut poll = Poll::new()?;
/// let mut events = Events::with_capacity(8);
///
/// // Create a new pipe.
/// let (mut sender, mut receiver) = new_pipe()?;
///
/// // Register both ends of the channel.
/// poll.registry().register(&mut receiver, PIPE_RECV, Interest::READABLE)?;
/// poll.registry().register(&mut sender, PIPE_SEND, Interest::WRITABLE)?;
///
/// const MSG: &[u8; 11] = b"Hello world";
///
/// loop {
///     poll.poll(&mut events, None)?;
///
///     for event in events.iter() {
///         match event.token() {
///             PIPE_SEND => sender.write(MSG)
///                 .and_then(|n| if n != MSG.len() {
///                         // We'll consider a short write an error in this
///                         // example. NOTE: we can't use `write_all` with
///                         // non-blocking I/O.
///                         Err(io::ErrorKind::WriteZero.into())
///                     } else {
///                         Ok(())
///                     })?,
///             PIPE_RECV => {
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
///
/// Example that receives an event once the `Sender` is dropped.
///
/// ```
/// # use std::io::{self, Read, Write};
/// #
/// # use mio::{Poll, Events, Interest, Token};
/// # use mio_pipe::new_pipe;
/// #
/// # const PIPE_RECV: Token = Token(0);
/// # const PIPE_SEND: Token = Token(1);
/// #
/// # fn main() -> io::Result<()> {
/// // Same setup as in the example above.
/// let mut poll = Poll::new()?;
/// let mut events = Events::with_capacity(8);
///
/// let (mut sender, mut receiver) = new_pipe()?;
///
/// poll.registry().register(&mut receiver, PIPE_RECV, Interest::READABLE)?;
/// poll.registry().register(&mut sender, PIPE_SEND, Interest::WRITABLE)?;
///
/// // Drop the sender.
/// drop(sender);
///
/// poll.poll(&mut events, None)?;
///
/// for event in events.iter() {
///     match event.token() {
///         PIPE_RECV if event.is_error() || event.is_read_closed() => {
///             // Detected that the sender was dropped.
///             println!("Sender dropped!");
///             return Ok(());
///         },
///         _ => unreachable!(),
///     }
/// }
/// # unreachable!();
/// # }
/// ```
pub fn new_pipe() -> io::Result<(Sender, Receiver)> {
    sys::new_pipe()
        .map(|(sender, receiver)| (Sender { inner: sender }, Receiver { inner: receiver }))
}
