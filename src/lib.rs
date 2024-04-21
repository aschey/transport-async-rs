#[cfg(feature = "codec")]
pub mod codec;
mod transport;
use tokio::io::{AsyncRead, AsyncWrite};
pub use transport::*;

pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Send + Unpin + 'static {}

impl<T> AsyncReadWrite for T where T: AsyncRead + AsyncWrite + Send + Unpin + 'static {}
