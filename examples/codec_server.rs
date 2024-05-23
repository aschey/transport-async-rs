use std::error::Error;

use clap::Parser;
use futures::{SinkExt, StreamExt};
use transport_async::codec::{Codec, CodecStream, SerdeCodec};
use transport_async::ipc::{OnConflict, SecurityAttributes, ServerId};
use transport_async::{ipc, tcp, udp, Bind, BoxedStream};

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
        TransportMode::Ipc => ipc::Endpoint::bind(ipc::EndpointParams::new(
            ServerId("test"),
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
        match result {
            Ok(mut stream) => {
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
            _ => unreachable!("ideally"),
        }
    }
    for conn in conns {
        conn.await?;
    }
    Ok(())
}
