use clap::Parser;
use futures::{SinkExt, StreamExt};
use parity_tokio_ipc::ServerId;
use std::error::Error;
use transport_async::codec::{Codec, SerdeCodec};
use transport_async::{ipc, tcp, udp, BoxedAsyncRW, Connect};

#[derive(clap::Parser)]
struct Cli {
    transport: TransportMode,
}
#[derive(clap::ValueEnum, Clone)]
enum TransportMode {
    Tcp,
    Udp,
    Ipc,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = Cli::parse();
    let client = match cli.transport {
        TransportMode::Tcp => tcp::Connection::connect("127.0.0.1:8081".parse()?)
            .await?
            .into_boxed(),
        TransportMode::Udp => udp::Connection::connect(udp::ConnectionParams {
            bind_addr: "127.0.0.1:9009".parse()?,
            connect_addr: "127.0.0.1:9010".parse()?,
        })
        .await?
        .into_boxed(),
        TransportMode::Ipc => {
            ipc::Connection::connect(ipc::ConnectionParams::new(ServerId("test"))?)
                .await?
                .into_boxed()
        }
    };

    let mut client = SerdeCodec::<usize, usize>::new(Codec::Bincode).client(client);

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
