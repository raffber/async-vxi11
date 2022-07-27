# Async VXI-11

Async VXI-11 library, supporting both `async-std` and `tokio`.

This implementation does not attempt to be a complete VXI-11 implementation but only implements the features the author(s) require. If you are missing a feature, please open an issue or a PR.

## Current Features

- Supports both `tokio` and `async-std`.
- Connect with TCP port mapper protocol
- Reading from and writing from an instrumnet

## Relevant RFC/Specifications

- XDR: <https://tools.ietf.org/html/rfc4506>
- ONC-RPC: <https://tools.ietf.org/html/rfc5531#section-9>
- PortMapper: <https://tools.ietf.org/html/rfc1833>
- VXI-11: <https://www.vxibus.org/specifications.html>

## License

Licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
