use std::net::SocketAddr;

use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;

use super::{Bind, Connect};

pub struct Endpoint {}

impl Bind for Endpoint {
    type Stream = TcpListenerStream;
    type Params = SocketAddr;

    async fn bind(params: Self::Params) -> io::Result<Self::Stream> {
        Ok(TcpListenerStream::new(TcpListener::bind(params).await?))
    }
}

pub struct Connection {}

impl Connect for Connection {
    type Stream = TcpStream;
    type Params = SocketAddr;

    async fn connect(params: Self::Params) -> io::Result<Self::Stream> {
        TcpStream::connect(params).await
    }
}
