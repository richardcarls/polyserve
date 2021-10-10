# polyserve

[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/clap-rs/clap/blob/master/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/clap-rs/clap/blob/master/LICENSE-MIT)
[![Contributors](https://img.shields.io/github/contributors/clap-rs/clap?style=flat-square)](https://github.com/clap-rs/clap/graphs/contributors)

Toy HTTP server built in Rust on top of [roa](https://github.com/Hexilee/roa).

It's main objective is to support advanced web server features purely with accessible filesystem-based configuration. This will be accomplished through a cascading flat-file configuration and naming conventions.

## Disclaimer
Don't actually use this for anything. Keep away from public-facing interfaces. The server listens on the loopback interface by default and is not configurable for this reason.

## Overview
What can `polyserve` do?
 - Asynchronously serve files from the web root
 - TLS connections
 - Cascading configuration via `.config.toml` files (analogous to Apache's .htaccess)
 - Indexes and Auto-Index support
 - Extension elision and trailing slash redirects
 - *Expose your system and all your secrets to the entire internet*

What's next?
 - Handlebars rendering (WIP):
    - layout templates
    - content templates
    - partials
    - script helpers
 - Markdown rendering
 - Tests
 - Queryable cascading context data, file frontmatter (SQLite?)
 - Cache layer
 - Get fat and grow a big white beard
 - Implement cool stuff like federated login, ActivityPub, etc.

## Installation

This project is not on crates.io yet! Clone this repo and build with rustc.

## Usage

### CLI

```bash
polyserve -p 3000 ./some-http-root
```

### Library
```toml
# Cargo.toml

[dependencies]
polyserve = { git = "https://github.com/richardcarls/polyserve/" }
async-std = { version = "1.10.0", features = ["attributes"] }
```

```rust,no_run
use polyserve::App;

#[async_std::main]
fn main() {
    let app = App::default();

    app.listen("127.0.0.1:8080", "./some-http-root").await?;
}
```

## Configuration
See `include/default.toml` for all options.

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