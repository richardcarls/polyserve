# polyserve

Toy HTTP server built in Rust.
It's main objective is to support advanced features like templated views, queryable data, federated services, and more with a purely file-based configuration. This will be accomplished through a combination of opinionated defaults with directory structure and file-naming conventions.

What can **polyserve** do?
 - Accept and handle incoming client connections in parallel
 - Parse incoming HTTP requests (v1.x only at this time, request body not currently supported)
 - Handle HTTP GET requests (other methods not yet supported)
 - Serve files statically (by resolving request URLs to file paths)
 - Support extension elision in request URLs (no extension required to resolve supported file types)
 - Support explicit directory indexes (in any supported file type)

What's next?
 - Render markdown files as HTML by default if extension is elided (configurable)
 - Custom templates for rendering (Handlebars)

## Installation

This project is not on crates.io yet! Clone this repo and build with rustc.

## Usage

### CLI

```bash
polyserve ./some-http-root
```

### Library

```rust
use polyserve::Server;

let mut server = Server::new();
// Call builder methods to set initial configuration

server.listen();
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

I am not looking for contributors at this time, but would love to hear feedback from anyone brave enough to use this.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.