// use std::error::Error;

// use futures::{SinkExt, StreamExt};
// use parity_tokio_ipc::{IpcSecurity, OnConflict, SecurityAttributes, ServerId};
// use transport_async::codec::{Codec, CodecStream, SerdeCodec};
// use transport_async::{ipc, Bind};

// #[tokio::main]
// pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
//     let incoming = ipc::Endpoint::bind(ipc::EndpointParams::new(
//         ServerId("test"),
//         SecurityAttributes::allow_everyone_create()?,
//         OnConflict::Overwrite,
//     )?)
//     .await?;

//     let mut transport = CodecStream::new(incoming, SerdeCodec::<usize, usize>::new(Codec::Bincode));
//     while let Some(result) = transport.next().await {
//         match result {
//             Ok(mut stream) => {
//                 tokio::spawn(async move {
//                     loop {
//                         if let Some(Ok(req)) = stream.next().await {
//                             println!("ping {req}");
//                             stream.send(req + 1).await.unwrap();
//                         }
//                     }
//                 });
//             }
//             _ => unreachable!("ideally"),
//         }
//     }

//     Ok(())
// }

use clap::Parser;
use futures::{SinkExt, StreamExt};
use parity_tokio_ipc::{IpcSecurity, OnConflict, SecurityAttributes, ServerId};
use std::error::Error;
use transport_async::codec::{Codec, CodecStream, SerdeCodec};
use transport_async::BoxedStream;
use transport_async::{ipc, tcp, udp, Bind};

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
    let incoming = match cli.transport {
        TransportMode::Tcp => tcp::Endpoint::bind("127.0.0.1:8081".parse()?)
            .await?
            .into_boxed(),
        TransportMode::Udp => udp::Endpoint::bind(udp::ConnectionParams {
            bind_addr: "127.0.0.1:9010".parse()?,
            connect_addr: "127.0.0.1:9009".parse()?,
        })
        .await?
        .into_boxed(),
        TransportMode::Ipc => ipc::Endpoint::bind(ipc::EndpointParams::new(
            ServerId("test"),
            SecurityAttributes::allow_everyone_create()?,
            OnConflict::Overwrite,
        )?)
        .await?
        .into_boxed(),
    };

    let mut transport = CodecStream::new(incoming, SerdeCodec::<usize, usize>::new(Codec::Bincode));

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
