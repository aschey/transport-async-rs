use std::{
    io,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::UdpSocket,
};

use crate::{Bind, Connect};

pub struct UdpStream {
    inner: UdpSocket,
}

pub struct UdpParams {
    pub bind_addr: SocketAddr,
    pub connect_addr: SocketAddr,
}

pub struct Endpoint {}

impl Bind for Endpoint {
    type Stream = tokio_stream::Once<io::Result<UdpStream>>;
    type Params = UdpParams;

    async fn bind(params: Self::Params) -> io::Result<Self::Stream> {
        let socket = UdpSocket::bind(params.bind_addr).await?;
        socket.connect(params.connect_addr).await?;
        Ok(tokio_stream::once(Ok(UdpStream { inner: socket })))
    }
}

impl AsyncRead for UdpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        self.inner.poll_recv(cx, buf)
    }
}

impl AsyncWrite for UdpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.inner.poll_send(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}

pub struct Connection {}

impl Connect for Connection {
    type Stream = UdpStream;
    type Params = UdpParams;

    async fn connect(params: Self::Params) -> io::Result<Self::Stream> {
        let socket = UdpSocket::bind(params.bind_addr).await?;
        socket.connect(params.connect_addr).await?;
        Ok(UdpStream { inner: socket })
    }
}
