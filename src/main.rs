use std::fs;
use std::sync::Arc;
use std::marker::Unpin;
use std::net::SocketAddr;
use std::time::Duration;
use std::path::PathBuf;

use futures::io::BufReader;
use futures::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt, AsyncBufRead, AsyncBufReadExt};
use futures::stream::StreamExt;
use async_std;
use async_std::net::{ToSocketAddrs, TcpListener};
use http;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = ("127.0.0.1", 3000)
        .to_socket_addrs().await?
        .find(|addr| addr.is_ipv4())
        .ok_or("No bind addr")?;

    let root_dir = PathBuf::from("./example-site")
        .canonicalize()?;

    println!("Serving {} at {}", root_dir.display(), addr);

    let ctx = Arc::new(RequestContext {
        addr,
        root_dir,
    });
    
    let tcp_listener = TcpListener::bind(addr).await?;
    
    tcp_listener
        .incoming()
        .for_each_concurrent(None, |stream| async {
            let stream = stream.unwrap();
            
            let ctx = Arc::clone(&ctx);
            
            async_std::task::spawn(async move {
                handle_connection(ctx, stream)
            });
        }).await;
    
    Ok(())
}

async fn handle_connection<S>(ctx: Arc<RequestContext>, mut stream: S)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut reader = BufReader::with_capacity(1024, stream);

    let mut lines = reader.lines();

    let mut parts = lines
        .next()
        .await?
        .unwrap()
        .split_whitespace();

    let mut request = http::Request::builder()
        .method(
            http::Method::from(parts.next().unwrap_or("GET"))
        )
        .version(
            http::Version::from(parts.next().unwrap_or("HTTP/1.1"))
        )
        .uri(
            http::Uri::from(parts.next().unwrap_or("/"))
        );
}

pub struct RequestContext {
    pub addr: SocketAddr,
    pub root_dir: PathBuf,
}


#[cfg(test)]
mod test {
    use std::fs;
    use std::cmp::min;
    use std::pin::Pin;
    use std::marker::Unpin;
    use std::task::{Context, Poll};
    use futures::{AsyncRead, AsyncWrite, AsyncReadExt};

    struct MockTcpStream {
        read_data: Vec<u8>,
        write_data: Vec<u8>,
    }

    impl AsyncRead for MockTcpStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            buf: &mut [u8]
        ) -> Poll<std::io::Result<usize>> {
            let size: usize = min(self.read_data.len(), buf.len());
            buf[..size]
                .copy_from_slice(&self.read_data[..size]);

            Poll::Ready(Ok(size))
        }
    }

    impl AsyncWrite for MockTcpStream {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
            buf: &[u8]
        ) -> Poll<std::io::Result<usize>> {
            self.write_data = Vec::from(buf);

            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _: &mut Context<'_>
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _: &mut Context<'_>
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    impl Unpin for MockTcpStream {}
    
    #[async_std::test]
    async fn test_handle_connection() {
        let input_bytes = b"GET / HTTP/1.1\r\n";

        let mut contents = vec![0u8; 1024];
        contents[..input_bytes.len()]
            .clone_from_slice(input_bytes);

        let mut stream = MockTcpStream {
            read_data: contents,
            write_data: Vec::new(),
        };

        super::handle_connection(&mut stream).await;

        let mut buf = [0u8; 1024];
        stream
            .read(&mut buf)
            .await
            .unwrap();

        let expected_contents = fs::read_to_string("hello.html").unwrap();
        let expected_response = format!("HTTP/1.1 200 OK\r\n\r\n{}", expected_contents);

        assert!(stream.write_data.starts_with(expected_response.as_bytes()));
    }
}
