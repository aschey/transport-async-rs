use std::io;

use futures::{Future, SinkExt, StreamExt, TryStream};
use tokio::io::{AsyncRead, AsyncWrite};
use transport_async::codec::{Codec, CodecStream, SerdeCodec};
use transport_async::{ipc, tcp, udp, Bind, Connect};

async fn run_server<I, S>(incoming: CodecStream<SerdeCodec<String, String>, I, io::Error, S>)
where
    I: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    S: TryStream<Ok = I, Error = io::Error>,
{
    futures::pin_mut!(incoming);
    while let Some(result) = incoming.next().await {
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
}

async fn run_clients<F, I, Fut>(create_client: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<I, io::Error>>,
    I: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    println!("Connecting to client 0...");
    let mut client_0 = SerdeCodec::<String, String>::new(Codec::Bincode)
        .client(create_client().await.expect("failed to open client_0"));

    println!("Connecting to client 1...");
    let mut client_1 = SerdeCodec::<String, String>::new(Codec::Bincode)
        .client(create_client().await.expect("failed to open client_0"));

    let msg = "hello".to_string();

    client_0
        .send(msg.clone())
        .await
        .expect("Unable to write message to client");
    let rx1 = client_0
        .next()
        .await
        .expect("Unable to read message from client")
        .unwrap();

    client_1
        .send(msg.clone())
        .await
        .expect("Unable to write message to client");
    let rx2 = client_1
        .next()
        .await
        .expect("Unable to read message from client")
        .unwrap();

    assert_eq!(rx1, msg);
    assert_eq!(rx2, msg);
}

async fn run_client<F, I, Fut>(create_client: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<I, io::Error>>,
    I: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    println!("Connecting to client 0...");
    let mut client_0 = SerdeCodec::<String, String>::new(Codec::Bincode)
        .client(create_client().await.expect("failed to open client_0"));

    let msg = "hello".to_string();

    client_0
        .send(msg.clone())
        .await
        .expect("Unable to write message to client");
    let rx1 = client_0
        .next()
        .await
        .expect("Unable to read message from client")
        .unwrap();

    assert_eq!(rx1, msg);
}

#[tokio::test]
async fn test_ipc() {
    let endpoint = ipc::Endpoint::bind(
        ipc::EndpointParams::new(
            ipc::ServerId("test"),
            ipc::SecurityAttributes::allow_everyone_create().unwrap(),
            ipc::OnConflict::Overwrite,
        )
        .unwrap(),
    )
    .await
    .unwrap();

    let transport = CodecStream::new(endpoint, SerdeCodec::<String, String>::new(Codec::Bincode));

    tokio::spawn(async move {
        run_server(transport).await;
    });
    run_clients(|| {
        ipc::Connection::connect(ipc::ConnectionParams::new(ipc::ServerId("test")).unwrap())
    })
    .await;
}

#[tokio::test]
async fn test_tcp() {
    let endpoint = tcp::Endpoint::bind("127.0.0.1:8081").await.unwrap();

    let transport = CodecStream::new(endpoint, SerdeCodec::<String, String>::new(Codec::Bincode));
    tokio::spawn(async move {
        run_server(transport).await;
    });
    run_clients(|| tcp::Connection::connect("127.0.0.1:8081")).await;
}

#[tokio::test]
async fn test_udp() {
    let endpoint = udp::Endpoint::bind(udp::ConnectionParams {
        bind_addr: "127.0.0.1:23683",
        connect_addr: "127.0.0.1:23684",
    })
    .await
    .unwrap();
    let transport = CodecStream::new(endpoint, SerdeCodec::<String, String>::new(Codec::Bincode));
    tokio::spawn(async move {
        run_server(transport).await;
    });
    run_client(|| {
        udp::Connection::connect(udp::ConnectionParams {
            bind_addr: "127.0.0.1:23684",
            connect_addr: "127.0.0.1:23683",
        })
    })
    .await;
}
