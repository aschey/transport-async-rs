use futures::{Future, TryStream};
use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "ipc")]
pub mod ipc;
#[cfg(feature = "local")]
pub mod local;
#[cfg(feature = "stdio")]
pub mod stdio;
#[cfg(feature = "tcp")]
pub mod tcp;
#[cfg(feature = "udp")]
pub mod udp;

pub trait Bind
where
    <<Self as Bind>::Stream as futures::TryStream>::Ok: AsyncRead + AsyncWrite,
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
