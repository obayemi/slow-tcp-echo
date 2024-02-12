use clap::Parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{debug, error, info};

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value = "127.0.0.1:8080")]
    addr: std::net::SocketAddr,
    #[clap(default_value_t = 2)]
    delay: u64,
    #[clap(short, long, default_value_t = 4096)]
    buffer_size: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let listener = TcpListener::bind(args.addr).await?;
    info!("Listening on: {}", args.addr);

    loop {
        let (mut socket, _) = listener.accept().await?;
        info!("receved new connection");

        tokio::spawn(async move {
            let mut buf = vec![0; args.buffer_size].into_boxed_slice();
            // Vec::with_capacity(args.buffer_size);
            loop {
                // In a loop, read data from the socket and write the data back.
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => {
                        info!("read 0 bytes, closing connection");
                        return;
                    }
                    Ok(n) => n,
                    Err(e) => {
                        error!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                info!("received: {n} bytes");
                debug!("read {n} bytes: {:?}", &buf[0..n]);
                tokio::time::sleep(tokio::time::Duration::from_secs(args.delay)).await;
                info!("slept {} sec, sending back {n} bytes", args.delay);
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    error!("failed to write to socket; err = {:?}", e);
                    return;
                };
            }
        });
    }
}
