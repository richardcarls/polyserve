#![windows_subsystem = "console"]

use std::path::PathBuf;

use clap::{Parser, ValueHint};

use polyserve::App;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    env_logger::init();

    let app = App::default();

    app.listen(
        (opts.interface, opts.port),
        opts.ipv4,
        opts.root.as_path(),
    ).await?;

    Ok(())
}

#[derive(Parser)]
#[clap(name = "polyserve", version, author, about)]
struct Opts {
    #[clap(
        long,
        help = "Only bind over IPv4.",
    )]
    ipv4: bool,
    
    // TODO: pnet crate, allow bind to literal interface by name
    #[clap(
        short,
        long,
        help = "IP address or hostname to bind.",
        default_value = "::1",
        default_value_if("ipv4", None, Some("127.0.0.1")),
    )]
    interface: String,

    /// Bind to port on interface
    #[clap(
        short,
        long,
        help = "TCP port number to bind.",
        default_value = "3000"
    )]
    port: u16,

    /// Web root to serve from
    #[clap(
        name = "ROOT",
        help = "Web root to serve.",
        default_value = ".",
        parse(from_os_str),
        value_hint = ValueHint::DirPath,
    )]
    root: PathBuf,
}
