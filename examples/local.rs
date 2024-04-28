use std::error::Error;
use std::result::Result;

use futures::{SinkExt, StreamExt};
use transport_async::local;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (mut transport, client_stream) = local::unbounded_channel::<usize, usize>();

    for _ in 0..2 {
        let mut client = client_stream.connect_unbounded().unwrap();
        tokio::spawn(async move {
            let mut next = 0;
            loop {
                println!("SEND: PING {next}");
                client
                    .send(next)
                    .await
                    .expect("Unable to write message to client");
                let val = client.next().await.expect("Unable to read buffer").unwrap();
                println!("RECEIVED: PONG {val}");
                next = val + 1;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    }

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
