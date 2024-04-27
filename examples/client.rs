use std::error::Error;

use clap::Parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use transport_async::ipc::ServerId;
use transport_async::{ipc, tcp, udp, BoxedAsyncRW, Connect};

#[derive(clap::Parser)]
struct Cli {
    transport: TransportMode,
}

#[derive(clap::ValueEnum, Clone)]
enum TransportMode {
    Tcp,
    Udp,
    Ipc,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = Cli::parse();
    let mut client = match cli.transport {
        TransportMode::Tcp => tcp::Connection::connect("127.0.0.1:8081".parse()?)
            .await?
            .into_boxed(),
        TransportMode::Udp => udp::Connection::connect(udp::ConnectionParams {
            bind_addr: "127.0.0.1:9009".parse()?,
            connect_addr: "127.0.0.1:9010".parse()?,
        })
        .await?
        .into_boxed(),
        TransportMode::Ipc => {
            ipc::Connection::connect(ipc::ConnectionParams::new(ServerId("test"))?)
                .await?
                .into_boxed()
        }
    };

    loop {
        let mut buf = [0u8; 4];
        println!("SEND: PING");
        client
            .write_all(b"ping")
            .await
            .expect("Unable to write message to client");
        client
            .read_exact(&mut buf[..])
            .await
            .expect("Unable to read buffer");
        if let Ok("pong") = std::str::from_utf8(&buf[..]) {
            println!("RECEIVED: PONG");
        } else {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    Ok(())
}
