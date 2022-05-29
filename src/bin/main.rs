#![windows_subsystem = "console"]

use std::error::Error;
use std::path::PathBuf;

use clap::{Parser, ValueHint};

use polyserve::App;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    simple_logger::init_with_level(log::Level::Info).unwrap();

    let app = App::default();

    app.listen(("127.0.0.1", opts.port), opts.root.as_path()).await?;

    Ok(())
}

#[derive(Parser)]
#[clap(name = "polyserve", version, author, about)]
#[clap()]
struct Opts {
    /// Bind to port on interface
    #[clap(short, long, default_value = "8080")]
    port: u16,

    /// Web root to serve from
    #[clap(
        name = "ROOT",
        default_value = ".",
        parse(from_os_str),
        value_hint = ValueHint::DirPath,
    )]
    root: PathBuf,
}
