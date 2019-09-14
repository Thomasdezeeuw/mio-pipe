//! Platform dependent implementation of the Unix pipe.

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use unix::{new_pipe, Receiver, Sender};

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::{new_pipe, Receiver, Sender};
