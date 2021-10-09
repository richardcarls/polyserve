#![windows_subsystem = "console"]

use std::error::Error;
use std::path::PathBuf;

use async_std;
use log;
use simple_logger;

use polyserve::App;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let app = App::default();

    let root = PathBuf::from("./example-site");

    app.listen("localhost:8080", root.as_path()).await?;

    Ok(())
}