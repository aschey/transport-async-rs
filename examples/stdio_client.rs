use std::error::Error;
use std::process::Stdio;

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use transport_async::Connect;
use transport_async::codec::{Codec, SerdeCodec};
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

    let (codec, codec_name) = match cli.codec {
        CodecMode::Bincode => (Codec::Bincode, "bincode"),
        CodecMode::Cbor => (Codec::Cbor, "cbor"),
        CodecMode::Json => (Codec::Json, "json"),
        CodecMode::MessagePack => (Codec::MessagePack, "message-pack"),
    };

    let process = tokio::process::Command::new("cargo")
        .args([
            "run",
            "--example",
            "stdio_server",
            "--all-features",
            codec_name,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let client = StdioTransport::connect(process)
        .await
        .expect("Process missing io handles");

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
