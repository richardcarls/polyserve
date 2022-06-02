use std::net::{ToSocketAddrs, TcpListener};

fn main() {
    match listen() {
        Ok(_) => 1,
        Err(err) => -1
    }
}

fn listen() -> Result<()> {
    let addr = ("127.0.0.1", 3000)
        .to_socket_addrs()?
        .find(|addr| addr.is_ipv4())
        .ok_or("No bind addr")?;

    let root_dir = PathBuf::from("./example-site")
        .canonicalize();

    println!("Serving {} at {}", root_dir.display(), addr);
    
    let tcp_listener = TcpListener::bind(addr)?;

    tcp_listener.incoming()
        .for_each(move |stream| {
            match stream {
                Ok(stream) => {
                    //
                },

                Err(err) => {
                    //
                }
            }
        });

    Ok(())
}
