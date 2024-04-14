use async_transport::codec::Codec;
use async_transport::codec::SerdeCodec;
use async_transport::transport::ipc;
use futures::{SinkExt, StreamExt};
use parity_tokio_ipc::ServerId;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client_transport = ipc::connect(ServerId("test")).await?;
    let mut codec = SerdeCodec::<usize, usize>::new(Codec::Bincode).client(client_transport);
    codec.send(1).await.unwrap();
    let res = codec.next().await.unwrap().unwrap();
    println!("pong {res}");
    Ok(())
}
