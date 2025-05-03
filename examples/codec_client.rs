use std::error::Error;
use std::fs;

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use transport_async::codec::{Codec, SerdeCodec};
use transport_async::ipc::ServerId;
use transport_async::{BoxedAsyncRW, Connect, ipc, quic, tcp, udp};

use crate::quic::rustls;
use crate::quic::rustls::RootCertStore;

#[derive(clap::Parser)]
struct Cli {
    transport: TransportMode,
    codec: CodecMode,
}

#[derive(clap::ValueEnum, Clone)]
enum TransportMode {
    Tcp,
    Udp,
    Ipc,
    Quic,
}

#[derive(clap::ValueEnum, Clone)]
enum CodecMode {
    Bincode,
    Cbor,
    Json,
    MessagePack,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = Cli::parse();
    let client = match cli.transport {
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
                let mut roots = RootCertStore::empty();
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

    let codec = match cli.codec {
        CodecMode::Bincode => Codec::Bincode,
        CodecMode::Cbor => Codec::Cbor,
        CodecMode::Json => Codec::Json,
        CodecMode::MessagePack => Codec::MessagePack,
    };

    let mut client = SerdeCodec::<usize, usize>::new(codec).client(client);

    let mut next = 0;
    loop {
        println!("SEND: PING {next}");
        client
            .send(next)
            .await
            .expect("Unable to write message to client");
        let val = client.next().await.expect("Unable to read buffer")?;
        println!("RECEIVED: PONG {val}");
        next = val + 1;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
