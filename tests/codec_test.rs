use std::fmt::Debug;
use std::io;
use std::pin::Pin;
use std::process::Stdio;

use bytes::{BufMut, Bytes, BytesMut};
use futures::{Future, SinkExt, Stream, StreamExt};
use transport_async::codec::{Codec, CodecStream, LengthDelimitedCodec, SerdeCodec, StreamSink};
use transport_async::stdio::StdioTransport;
use transport_async::{ipc, local, tcp, udp, Bind, Connect};

async fn run_server<I, E, S>(stream: Pin<Box<dyn Stream<Item = Result<I, E>> + Send>>)
where
    I: StreamSink<S, Error = E, Item = Result<S, E>>,
    E: Debug,
{
    futures::pin_mut!(stream);
    while let Some(result) = stream.next().await {
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

async fn run_server_bytes<I, E>(stream: Pin<Box<dyn Stream<Item = Result<I, E>> + Send>>)
where
    I: StreamSink<Bytes, Error = E, Item = Result<BytesMut, E>>,
    E: Debug,
{
    futures::pin_mut!(stream);
    while let Some(result) = stream.next().await {
        match result {
            Ok(mut stream) => {
                let msg = stream.next().await.expect("unable to read from socket");
                stream
                    .send(msg.unwrap().freeze())
                    .await
                    .expect("unable to write to socket");
            }
            _ => unreachable!("ideally"),
        }
    }
}

async fn run_clients<F, I, Fut, E>(create_client: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<I, io::Error>>,
    I: StreamSink<String, Error = E, Item = Result<String, E>>,
    E: Debug,
{
    println!("Connecting to client 0...");
    let mut client_0 = create_client().await.expect("failed to open client_0");

    println!("Connecting to client 1...");
    let mut client_1 = create_client().await.expect("failed to open client_0");

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

async fn run_clients_bytes<F, I, Fut, E>(create_client: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<I, io::Error>>,
    I: StreamSink<Bytes, Error = E, Item = Result<BytesMut, E>>,
    E: Debug,
{
    println!("Connecting to client 0...");
    let mut client_0 = create_client().await.expect("failed to open client_0");

    println!("Connecting to client 1...");
    let mut client_1 = create_client().await.expect("failed to open client_0");

    let mut msg = BytesMut::new();
    msg.put(&b"hello"[..]);

    client_0
        .send(msg.clone().freeze())
        .await
        .expect("Unable to write message to client");
    let rx1 = client_0
        .next()
        .await
        .expect("Unable to read message from client")
        .unwrap();

    client_1
        .send(msg.clone().freeze())
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

async fn run_client<F, I, Fut, E>(create_client: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<I, E>>,
    I: StreamSink<String, Error = E, Item = Result<String, E>>,
    E: Debug,
{
    println!("Connecting to client 0...");
    let mut client_0 = create_client().await.expect("failed to open client_0");
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
        run_server(transport.boxed()).await;
    });
    run_clients(|| async move {
        let client =
            ipc::Connection::connect(ipc::ConnectionParams::new(ipc::ServerId("test"))?).await?;
        Ok(SerdeCodec::<String, String>::new(Codec::Bincode).client(client))
    })
    .await;
}

#[tokio::test]
async fn test_tcp() {
    let endpoint = tcp::Endpoint::bind("127.0.0.1:8081").await.unwrap();

    let transport = CodecStream::new(endpoint, SerdeCodec::<String, String>::new(Codec::Bincode));
    tokio::spawn(async move {
        run_server(transport.boxed()).await;
    });
    run_clients(|| async move {
        let client = tcp::Connection::connect("127.0.0.1:8081").await?;
        Ok(SerdeCodec::<String, String>::new(Codec::Bincode).client(client))
    })
    .await;
}

#[tokio::test]
async fn length_delimited() {
    let endpoint = tcp::Endpoint::bind("127.0.0.1:8081").await.unwrap();

    let transport = CodecStream::new(endpoint, LengthDelimitedCodec);
    tokio::spawn(async move {
        run_server_bytes(transport.boxed()).await;
    });
    run_clients_bytes(|| async move {
        let client = tcp::Connection::connect("127.0.0.1:8081").await?;
        Ok(LengthDelimitedCodec::client(client))
    })
    .await;
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
        run_server(transport.boxed()).await;
    });
    run_client(|| async move {
        let client = udp::Connection::connect(udp::ConnectionParams {
            bind_addr: "127.0.0.1:23684",
            connect_addr: "127.0.0.1:23683",
        })
        .await?;
        Ok(SerdeCodec::<String, String>::new(Codec::Bincode).client(client))
    })
    .await;
}

#[tokio::test]
async fn test_local() {
    let (transport, client_stream) = local::unbounded_channel::<String, String>();
    tokio::spawn(async move {
        run_server(transport.boxed()).await;
    });
    run_client(|| async { client_stream.connect_unbounded() }).await;
}

#[tokio::test]
async fn test_local_bounded() {
    let (transport, client_stream) = local::channel::<String, String>(32);
    tokio::spawn(async move {
        run_server(transport.boxed()).await;
    });
    run_client(|| async { client_stream.connect_bounded(32) }).await;
}

#[tokio::test]
async fn test_stdio() {
    let process = tokio::process::Command::new("cargo")
        .args(["run", "--example", "stdio_test", "--all-features"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    run_client(|| async move {
        let client = StdioTransport::connect(process).await.unwrap();
        Ok(SerdeCodec::<String, String>::new(Codec::Bincode).client(client))
    })
    .await;
}
