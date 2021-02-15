# async-jsonrpc-client

[![ga-svg]][ga-url]
[![crates-svg]][crates-url]
[![docs-svg]][docs-url]

[ga-svg]: https://github.com/koushiro/async-jsonrpc/workflows/build/badge.svg
[ga-url]: https://github.com/koushiro/async-jsonrpc/actions
[crates-svg]: https://img.shields.io/crates/v/async-jsonrpc-client
[crates-url]: https://crates.io/crates/async-jsonrpc-client
[docs-svg]: https://docs.rs/async-jsonrpc-client/badge.svg
[docs-url]: https://docs.rs/async-jsonrpc-client

An async JSON-RPC 2.0 client library written in Rust.

## Features

- support HTTP
- support WebSocket
- support batch request
- support subscription (only for WebSocket client)
- support `async-std` and `tokio` runtime

## Usage

See the [examples](examples) for details.

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
