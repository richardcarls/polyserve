use clap::Clap;
use cascade::cascade;

use polyserve::{ Server, ServerError };

#[derive(Debug, Clap)]
#[clap(name = "polyserve")]
struct ServerOpt {
    #[clap(
        name = "ipv4",
        long,
        about = "Only bind over IPv4",
    )]
    ipv4: bool,
    
    // TODO: pnet crate, allow binf to literal interface by name
    #[clap(
        name = "interface",
        short,
        long,
        about = "IP address or hostname to bind to.",
        default_value = "::",
        default_value_if("ipv4", None, "0.0.0.0"),
    )]
    interface: String,

    #[clap(
        name = "port",
        short,
        long,
        about = "The TCP port to listen on.",
        default_value = "8080",
    )]
    port: u16,

    #[clap(
        name = "root",
        about = "The root directory of the server.",
    )]
    root: Option<String>,
}

fn main() {
    std::process::exit(match run_server() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Server Error: {:?}", err);
            1
        }
    });
}

fn run_server() -> Result<(), ServerError> {
    let opt = ServerOpt::parse();

    let mut server = cascade! {
        Server::new();
        ..set_ipv4(opt.ipv4);
        ..with_interface(opt.interface.as_str());
        ..with_port(opt.port);
    };

    if let Some(root) = opt.root {
        server.with_root(root.as_str());
    }

    server.listen()?;

    Ok(())
}