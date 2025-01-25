use std::error::Error;
use std::fs;

use clap::Parser;
use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt, split};
use transport_async::ipc::{OnConflict, SecurityAttributes, ServerId};
use transport_async::{Bind, BoxedStream, ipc, quic, tcp, udp};

use crate::quic::rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

#[derive(clap::Parser)]
struct Cli {
    transport: TransportMode,
}

#[derive(clap::ValueEnum, Clone)]
enum TransportMode {
    Tcp,
    Udp,
    Quic,
    Ipc,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = Cli::parse();
    let incoming = match cli.transport {
        TransportMode::Tcp => tcp::Endpoint::bind("127.0.0.1:8081").await?.into_boxed(),
        TransportMode::Udp => udp::Endpoint::bind(udp::ConnectionParams {
            bind_addr: "127.0.0.1:9010",
            connect_addr: "127.0.0.1:9009",
        })
        .await?
        .into_boxed(),
        TransportMode::Quic => quic::Endpoint::bind(quic::EndpointParams {
            addr: "127.0.0.1:8081".parse().unwrap(),
            tls_server_config: {
                let cert = CertificateDer::from(fs::read("./examples/certs/cert.der").unwrap());
                let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                    fs::read("./examples/certs/cert.key").unwrap(),
                ));

                let mut config = quinn::rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(vec![cert], key)
                    .unwrap();
                config.alpn_protocols = vec![b"hq-29".into()];
                config
            },
        })
        .await?
        .into_boxed(),
        TransportMode::Ipc => ipc::Endpoint::bind(ipc::EndpointParams::new(
            ServerId::new("test"),
            SecurityAttributes::allow_everyone_create()?,
            OnConflict::Overwrite,
        )?)
        .await?
        .into_boxed(),
    };

    futures::pin_mut!(incoming);
    let mut conns = vec![];
    while let Some(result) = incoming.next().await {
        if let Ok(stream) = result {
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
    }
    for conn in conns {
        conn.await?;
    }
    Ok(())
}
