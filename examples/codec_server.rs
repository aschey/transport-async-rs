use std::error::Error;
use std::fs;

use clap::Parser;
use futures::{SinkExt, StreamExt};
use transport_async::codec::{Codec, CodecStream, SerdeCodec};
use transport_async::ipc::{OnConflict, SecurityAttributes, ServerId};
use transport_async::{ipc, quic, tcp, udp, Bind, BoxedStream};

use crate::quic::rustls;
use crate::quic::rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

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

                let mut config = rustls::ServerConfig::builder()
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

    let codec = match cli.codec {
        CodecMode::Bincode => Codec::Bincode,
        CodecMode::Cbor => Codec::Cbor,
        CodecMode::Json => Codec::Json,
        CodecMode::MessagePack => Codec::MessagePack,
    };

    let mut transport = CodecStream::new(incoming, SerdeCodec::<usize, usize>::new(codec));

    let mut conns = vec![];
    while let Some(result) = transport.next().await {
        if let Ok(mut stream) = result {
            conns.push(tokio::spawn(async move {
                loop {
                    if let Some(Ok(val)) = stream.next().await {
                        println!("RECEIVED: PING {val}");
                        let next = val + 1;
                        println!("SEND: PONG {next}");
                        stream.send(next).await.expect("failed to send");
                    } else {
                        println!("Closing socket");
                        break;
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
