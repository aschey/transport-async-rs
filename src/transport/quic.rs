use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

use async_stream::stream;
use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;
use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
pub use quinn::rustls;
use quinn::{ClientConfig, RecvStream, SendStream};
use tokio::io::{self, AsyncRead, AsyncWrite};

use super::{Bind, Connect};

pub struct EndpointParams {
    pub addr: SocketAddr,
    pub tls_server_config: rustls::ServerConfig,
}

pub struct Endpoint {}

impl Bind for Endpoint {
    type Stream = QuicListenerStream;
    type Params = EndpointParams;

    async fn bind(params: Self::Params) -> io::Result<Self::Stream> {
        let server_config = quinn::ServerConfig::with_crypto(Arc::new(
            QuicServerConfig::try_from(params.tls_server_config)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?,
        ));

        let endpoint = quinn::Endpoint::server(server_config, params.addr)?;
        let stream = stream! {
            while let Some(incoming) = endpoint.accept().await {
                let conn = incoming.await?;
                let (send, recv) = conn.accept_bi().await?;
                yield Ok::<_, io::Error>(QuicStream { send, recv })
            }
        }
        .boxed();
        Ok(QuicListenerStream { inner: stream })
    }
}

pub struct ConnectionParams {
    pub connect_addr: SocketAddr,
    pub bind_addr: SocketAddr,
    pub tls_config: rustls::ClientConfig,
    pub server_name: String,
}
pub struct Connection {}

impl Connect for Connection {
    type Stream = QuicStream;
    type Params = ConnectionParams;

    async fn connect(params: Self::Params) -> io::Result<Self::Stream> {
        let mut endpoint = quinn::Endpoint::client(params.bind_addr)?;

        endpoint.set_default_client_config(ClientConfig::new(Arc::new(
            QuicClientConfig::try_from(params.tls_config)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?,
        )));

        let connection = endpoint
            .connect(params.connect_addr, &params.server_name)
            .map_err(|e| io::Error::new(io::ErrorKind::ConnectionRefused, e.to_string()))?
            .await?;
        let (send, recv) = connection.open_bi().await?;
        Ok(QuicStream { send, recv })
    }
}

pin_project! {
    pub struct QuicStream {
        #[pin]
        send: SendStream,
        #[pin]
        recv: RecvStream,
    }
}

impl AsyncRead for QuicStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.project().recv.poll_read(cx, buf)
    }
}

impl AsyncWrite for QuicStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        tokio::io::AsyncWrite::poll_write(self.project().send, cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        self.project().send.poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.project().send.poll_shutdown(cx)
    }
}

pin_project! {
    pub struct QuicListenerStream {
        #[pin]
        inner: Pin<Box<dyn Stream<Item = Result<QuicStream, io::Error>> + Send>>,
    }
}

impl Stream for QuicListenerStream {
    type Item = Result<QuicStream, io::Error>;
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}
