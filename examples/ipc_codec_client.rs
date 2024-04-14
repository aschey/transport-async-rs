use futures::{SinkExt, StreamExt};
use parity_tokio_ipc::ServerId;
use std::error::Error;
use transport_async::codec::Codec;
use transport_async::codec::SerdeCodec;
use transport_async::transport::ipc;
use transport_async::transport::Connect;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = ipc::Connection::connect(ipc::ConnectionParams::new(ServerId("test"))?).await?;
    let mut client = SerdeCodec::<usize, usize>::new(Codec::Bincode).client(client);
    client.send(1).await.unwrap();
    let res = client.next().await.unwrap().unwrap();
    println!("pong {res}");
    Ok(())
}
