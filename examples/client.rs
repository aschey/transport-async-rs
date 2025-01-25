use std::error::Error;
use std::fs;

use clap::Parser;
use quinn::rustls;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use transport_async::ipc::ServerId;
use transport_async::{BoxedAsyncRW, Connect, ipc, quic, tcp, udp};

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
    let mut client = match cli.transport {
        TransportMode::Tcp => tcp::Connection::connect("127.0.0.1:8081")
            .await?
            .into_boxed(),
        TransportMode::Udp => udp::Connection::connect(udp::ConnectionParams {
            bind_addr: "127.0.0.1:9009",
            connect_addr: "127.0.0.1:9010",
        })
        .await?
        .into_boxed(),
        TransportMode::Quic => quic::Connection::connect(quic::ConnectionParams {
            bind_addr: "0.0.0.0:0".parse().unwrap(),
            connect_addr: "127.0.0.1:8081".parse().unwrap(),
            server_name: "localhost".to_string(),
            tls_config: {
                let mut roots = rustls::RootCertStore::empty();
                roots
                    .add(quinn::rustls::pki_types::CertificateDer::from(
                        fs::read("./examples/certs/cert.der").unwrap(),
                    ))
                    .unwrap();
                let mut client_crypto = rustls::ClientConfig::builder()
                    .with_root_certificates(roots)
                    .with_no_client_auth();

                client_crypto.alpn_protocols = vec![b"hq-29".into()];
                client_crypto
            },
        })
        .await?
        .into_boxed(),
        TransportMode::Ipc => {
            ipc::Connection::connect(ipc::ConnectionParams::new(ServerId::new("test"))?)
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
