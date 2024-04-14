use futures::{SinkExt, StreamExt};
use parity_tokio_ipc::ServerId;
use std::error::Error;
use transport_async::codec::Codec;
use transport_async::codec::SerdeCodec;
use transport_async::transport::ipc;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client_transport = ipc::connect(ServerId("test")).await?;
    let mut codec = SerdeCodec::<usize, usize>::new(Codec::Bincode).client(client_transport);
    codec.send(1).await.unwrap();
    let res = codec.next().await.unwrap().unwrap();
    println!("pong {res}");
    Ok(())
}
