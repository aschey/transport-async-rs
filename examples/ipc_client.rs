use std::error::Error;

use parity_tokio_ipc::ServerId;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use transport_async::{ipc, Connect};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client =
        ipc::Connection::connect(ipc::ConnectionParams::new(ServerId("test"))?).await?;

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
