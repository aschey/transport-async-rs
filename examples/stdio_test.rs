use std::error::Error;

use futures_util::{SinkExt, StreamExt};
use transport_async::codec::{Codec, CodecStream, SerdeCodec};
use transport_async::stdio::StdioTransport;

// This is just used for the stdio integration tests

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let incoming = StdioTransport::incoming();
    let mut transport =
        CodecStream::new(incoming, SerdeCodec::<String, String>::new(Codec::Bincode));
    while let Some(result) = transport.next().await {
        match result {
            Ok(mut stream) => {
                let msg = stream.next().await.expect("unable to read from socket");
                stream
                    .send(msg.unwrap())
                    .await
                    .expect("unable to write to socket");
            }
            _ => unreachable!("ideally"),
        }
    }
    Ok(())
}
