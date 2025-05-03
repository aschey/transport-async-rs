use std::io;
use std::pin::Pin;

use futures_util::{Future, Stream, StreamExt, TryStream};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::AsyncReadWrite;
#[cfg(feature = "ipc")]
pub mod ipc;
#[cfg(feature = "local")]
pub mod local;
#[cfg(feature = "quic")]
pub mod quic;
#[cfg(feature = "stdio")]
pub mod stdio;
#[cfg(feature = "tcp")]
pub mod tcp;
#[cfg(feature = "udp")]
pub mod udp;

pub trait Bind
where
    <<Self as Bind>::Stream as futures_util::TryStream>::Ok: AsyncRead + AsyncWrite,
{
    type Stream: TryStream;
    type Params;

    fn bind(params: Self::Params) -> impl Future<Output = io::Result<Self::Stream>> + Send;
}

pub trait Connect {
    type Stream: AsyncRead + AsyncWrite;
    type Params;

    fn connect(params: Self::Params) -> impl Future<Output = io::Result<Self::Stream>> + Send;
}

pub type AsyncRwStream = dyn Stream<Item = io::Result<Box<dyn AsyncReadWrite>>>;

pub trait BoxedStream {
    fn into_boxed(self) -> Pin<Box<AsyncRwStream>>;
}

impl<T, S> BoxedStream for T
where
    T: Stream<Item = io::Result<S>> + 'static,
    S: AsyncReadWrite,
{
    fn into_boxed(self) -> Pin<Box<AsyncRwStream>> {
        Box::pin(self.map(|s| s.map(|s| Box::new(s) as Box<dyn AsyncReadWrite>)))
    }
}

pub trait BoxedAsyncRW {
    fn into_boxed(self) -> Pin<Box<dyn AsyncReadWrite>>;
}

impl<T> BoxedAsyncRW for T
where
    T: AsyncReadWrite,
{
    fn into_boxed(self) -> Pin<Box<dyn AsyncReadWrite>> {
        Box::pin(self)
    }
}
