use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use pin_project::pin_project;
use tokio::io;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_stream::wrappers::TcpListenerStream;

pub type TcpStream = tokio::net::TcpStream;

pub async fn create_endpoint(addr: SocketAddr) -> io::Result<TcpListenerStream> {
    TcpTransport::bind(addr).await
}

#[pin_project]
pub struct TcpTransport {
    #[pin]
    listener: TcpListener,
}

impl TcpTransport {
    async fn bind(addr: SocketAddr) -> io::Result<TcpListenerStream> {
        Ok(TcpListenerStream::new(TcpListener::bind(addr).await?))
    }
}

// impl Stream for TcpTransport {
//     type Item = Result<TcpStream, io::Error>;

//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         match self.project().listener.poll_accept(cx) {
//             Poll::Ready(Ok((stream, _))) => Poll::Ready(Some(Ok(stream))),
//             Poll::Ready(Err(_)) => Poll::Ready(None),
//             Poll::Pending => Poll::Pending,
//         }
//     }
// }

pub async fn connect(addr: impl ToSocketAddrs) -> io::Result<TcpStream> {
    TcpStream::connect(addr).await
}
