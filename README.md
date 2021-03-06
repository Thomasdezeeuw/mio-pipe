# Mio pipe

[![Build Status](https://travis-ci.com/Thomasdezeeuw/mio-pipe.svg?branch=master)](https://travis-ci.com/Thomasdezeeuw/mio-pipe)
[![Build status](https://api.cirrus-ci.com/github/Thomasdezeeuw/mio-pipe.svg)](https://cirrus-ci.com/github/Thomasdezeeuw/mio-pipe)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/mio-pipe.svg)](https://crates.io/crates/mio-pipe)
[![Docs](https://docs.rs/mio-pipe/badge.svg)](https://docs.rs/mio-pipe)

Crate that wraps a Unix pipe for use with [Mio].

See the [API documentation] for more.

[Mio]: https://crates.io/crates/mio
[API documentation]: https://docs.rs/mio-pipe

## Deprecation notice

Since version 0.7.5 Mio has support for the `pipe(2)` system call in the crate
(based on this implementation):
https://docs.rs/mio/0.7.5/mio/unix/pipe/index.html.


## Supported platforms

Currently supported platforms:

* Android
* DragonFly BSD
* FreeBSD
* Linux
* NetBSD
* OpenBSD
* iOS
* macOS

The most notable exception in the list is Windows. If you want to contribute a
port to Windows please see [issue #6].

[issue #6]: https://github.com/Thomasdezeeuw/mio-pipe/issues/6


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed as above, without any
additional terms or conditions.
