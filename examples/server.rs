use clap::Parser;
use futures::StreamExt;
use parity_tokio_ipc::{IpcSecurity, OnConflict, SecurityAttributes, ServerId};
use std::error::Error;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
use transport_async::BoxedStream;
use transport_async::{ipc, tcp, udp, Bind};

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
    let incoming = match cli.transport {
        TransportMode::Tcp => tcp::Endpoint::bind("127.0.0.1:8081".parse()?)
            .await?
            .into_boxed(),
        TransportMode::Udp => udp::Endpoint::bind(udp::ConnectionParams {
            bind_addr: "127.0.0.1:9010".parse()?,
            connect_addr: "127.0.0.1:9009".parse()?,
        })
        .await?
        .into_boxed(),
        TransportMode::Ipc => ipc::Endpoint::bind(ipc::EndpointParams::new(
            ServerId("test"),
            SecurityAttributes::allow_everyone_create()?,
            OnConflict::Overwrite,
        )?)
        .await?
        .into_boxed(),
    };

    futures::pin_mut!(incoming);
    let mut conns = vec![];
    while let Some(result) = incoming.next().await {
        match result {
            Ok(stream) => {
                let (mut reader, mut writer) = split(stream);

                conns.push(tokio::spawn(async move {
                    loop {
                        let mut buf = [0u8; 4];

                        if reader.read_exact(&mut buf).await.is_err() {
                            println!("Closing socket");
                            break;
                        }
                        if let Ok("ping") = std::str::from_utf8(&buf[..]) {
                            println!("RECEIVED: PING");
                            writer
                                .write_all(b"pong")
                                .await
                                .expect("unable to write to socket");
                            println!("SEND: PONG");
                        }
                    }
                }));
            }
            _ => unreachable!("ideally"),
        }
    }
    for conn in conns {
        conn.await?;
    }
    Ok(())
}
