use std::error::Error;

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use transport_async::codec::{Codec, CodecStream, SerdeCodec};
use transport_async::stdio::StdioTransport;

#[derive(clap::Parser)]
struct Cli {
    codec: CodecMode,
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

    let codec = match cli.codec {
        CodecMode::Bincode => Codec::Bincode,
        CodecMode::Cbor => Codec::Cbor,
        CodecMode::Json => Codec::Json,
        CodecMode::MessagePack => Codec::MessagePack,
    };
    let incoming = StdioTransport::incoming();
    let mut transport = CodecStream::new(incoming, SerdeCodec::<usize, usize>::new(codec));

    let mut conns = vec![];
    while let Some(result) = transport.next().await {
        match result {
            Ok(mut stream) => {
                conns.push(tokio::spawn(async move {
                    while let Some(Ok(val)) = stream.next().await {
                        let next = val + 1;
                        stream.send(next).await.expect("failed to send");
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
