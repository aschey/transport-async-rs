use futures::{Future, TryStream};
use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
pub mod codec;
pub mod ipc;
pub mod local;
pub mod stdio;
pub mod tcp;

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
