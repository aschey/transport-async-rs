use async_transport::transport::ipc;
use futures::StreamExt;
use parity_tokio_ipc::{IpcSecurity, OnConflict, SecurityAttributes, ServerId};
use std::error::Error;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let incoming = ipc::create_endpoint(
        ServerId("test"),
        SecurityAttributes::allow_everyone_create().expect("Failed to set security attributes"),
        OnConflict::Overwrite,
    )?;

    futures::pin_mut!(incoming);

    while let Some(result) = incoming.next().await {
        match result {
            Ok(stream) => {
                let (mut reader, mut writer) = split(stream);

                tokio::spawn(async move {
                    loop {
                        let mut buf = [0u8; 4];

                        if reader.read_exact(&mut buf).await.is_err() {
                            println!("Closing socket");
                            break;
                        }
                        if let Ok("ping") = std::str::from_utf8(&buf[..]) {
                            println!("RECEIVED: PING");
                            writer
                                .write_all(b"pong")
                                .await
                                .expect("unable to write to socket");
                            println!("SEND: PONG");
                        }
                    }
                });
            }
            _ => unreachable!("ideally"),
        }
    }
    Ok(())
}
