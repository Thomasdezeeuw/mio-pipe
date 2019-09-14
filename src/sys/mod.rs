//! Platform dependent implementation of the Unix pipe.

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use unix::{new_pipe, Receiver, Sender};

// TODO: add Windows implementation.
