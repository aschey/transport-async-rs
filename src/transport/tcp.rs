use std::marker::PhantomData;

use tokio::io;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_stream::wrappers::TcpListenerStream;

use super::{Bind, Connect};

pub struct Endpoint<A> {
    _phantom: PhantomData<A>,
}

impl<A> Bind for Endpoint<A>
where
    A: ToSocketAddrs + Send,
{
    type Stream = TcpListenerStream;
    type Params = A;

    async fn bind(params: Self::Params) -> io::Result<Self::Stream> {
        Ok(TcpListenerStream::new(TcpListener::bind(params).await?))
    }
}

pub struct Connection<A> {
    _phantom: PhantomData<A>,
}

impl<A> Connect for Connection<A>
where
    A: ToSocketAddrs + Send,
{
    type Stream = TcpStream;
    type Params = A;

    async fn connect(params: Self::Params) -> io::Result<Self::Stream> {
        TcpStream::connect(params).await
    }
}
