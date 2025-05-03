use std::fmt::Debug;
use std::io;

use futures_util::{Future, Stream, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt, split};
use transport_async::local::{self, LocalClientStream};
use transport_async::{AsyncReadWrite, Bind, Connect, ipc, tcp, udp};

async fn run_server<S, A, E>(incoming: S)
where
    S: Stream<Item = Result<A, E>>,
    A: AsyncReadWrite,
{
    futures_util::pin_mut!(incoming);
    while let Some(result) = incoming.next().await {
        match result {
            Ok(stream) => {
                let (mut reader, mut writer) = split(stream);
                let mut buf = [0u8; 5];
                reader
                    .read_exact(&mut buf)
                    .await
                    .expect("unable to read from socket");
                writer
                    .write_all(&buf[..])
                    .await
                    .expect("unable to write to socket");
            }
            _ => unreachable!("ideally"),
        }
    }
}

async fn run_clients<F, A, Fut>(create_client: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<A, io::Error>>,
    A: AsyncReadWrite,
{
    println!("Connecting to client 0...");
    let mut client_0 = create_client().await.expect("failed to open client_0");

    println!("Connecting to client 1...");
    let mut client_1 = create_client().await.expect("failed to open client_1");
    let msg = b"hello";

    let mut rx_buf = vec![0u8; msg.len()];
    client_0
        .write_all(msg)
        .await
        .expect("Unable to write message to client");
    client_0
        .read_exact(&mut rx_buf)
        .await
        .expect("Unable to read message from client");

    let mut rx_buf2 = vec![0u8; msg.len()];
    client_1
        .write_all(msg)
        .await
        .expect("Unable to write message to client");
    client_1
        .read_exact(&mut rx_buf2)
        .await
        .expect("Unable to read message from client");

    assert_eq!(rx_buf, msg);
    assert_eq!(rx_buf2, msg);
}

async fn run_client<F, A, Fut, E>(create_client: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<A, E>>,
    A: AsyncReadWrite,
    E: Debug,
{
    println!("Connecting to client 0...");
    let mut client_0 = create_client().await.expect("failed to open client_0");

    let msg = b"hello";

    let mut rx_buf = vec![0u8; msg.len()];
    client_0
        .write_all(msg)
        .await
        .expect("Unable to write message to client");
    client_0
        .read_exact(&mut rx_buf)
        .await
        .expect("Unable to read message from client");

    assert_eq!(rx_buf, msg);
}

#[tokio::test]
async fn test_ipc() {
    let endpoint = ipc::Endpoint::bind(
        ipc::EndpointParams::new(
            ipc::ServerId::new("test"),
            ipc::SecurityAttributes::allow_everyone_create().unwrap(),
            ipc::OnConflict::Overwrite,
        )
        .unwrap(),
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        run_server(endpoint).await;
    });
    run_clients(|| {
        ipc::Connection::connect(ipc::ConnectionParams::new(ipc::ServerId::new("test")).unwrap())
    })
    .await;
}

#[tokio::test]
async fn test_tcp() {
    let endpoint = tcp::Endpoint::bind("127.0.0.1:52542").await.unwrap();

    tokio::spawn(async move {
        run_server(endpoint).await;
    });
    run_clients(|| tcp::Connection::connect("127.0.0.1:52542")).await;
}

#[tokio::test]
async fn test_udp() {
    let endpoint = udp::Endpoint::bind(udp::ConnectionParams {
        bind_addr: "127.0.0.1:26432",
        connect_addr: "127.0.0.1:26433",
    })
    .await
    .unwrap();

    tokio::spawn(async move {
        run_server(endpoint).await;
    });
    run_client(|| {
        udp::Connection::connect(udp::ConnectionParams {
            bind_addr: "127.0.0.1:26433",
            connect_addr: "127.0.0.1:26432",
        })
    })
    .await;
}

#[tokio::test]
async fn test_local() {
    let (transport, client_stream) = local::unbounded_channel::<Vec<u8>, Vec<u8>>();
    tokio::spawn(async move {
        run_server(transport).await;
    });
    run_client(|| LocalClientStream::connect(client_stream)).await;
}

#[tokio::test]
async fn test_local_bounded() {
    let (transport, client_stream) = local::channel::<Vec<u8>, Vec<u8>>(32);
    tokio::spawn(async move {
        run_server(transport).await;
    });
    run_client(move || LocalClientStream::connect(client_stream)).await;
}
