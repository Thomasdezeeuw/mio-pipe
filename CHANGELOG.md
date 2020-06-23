# 0.1.1

* Update to Mio v0.7.0 (from 0.7.0-alpha.1).
* Add `Sender::set_nonblocking` and `Receiver::set_nonblocking`.
* Set `FD_CLOEXEC` on MacOS, iOS and Solaris.
* Implement `FromRawFd` for `Sender` and `Receiver`.

# 0.1.0

Initial release.
