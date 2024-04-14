use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{ready, Stream, TryStream};
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::codec::{CodecBuilder, CodecStream};

#[pin_project]
pub struct CodecTransport<B, I, E, S>
where
    B: CodecBuilder,
    S: TryStream<Ok = I, Error = E>,
    I: AsyncRead + AsyncWrite,
{
    #[pin]
    codec_builder: B,
    #[pin]
    inner: S,
}

impl<B, I, E, S> CodecTransport<B, I, E, S>
where
    B: CodecBuilder,
    S: TryStream<Ok = I, Error = E>,
    I: AsyncRead + AsyncWrite,
{
    pub fn new(inner: S, codec_builder: B) -> Self {
        Self {
            inner,
            codec_builder,
        }
    }
}

impl<B, I, E, S> Stream for CodecTransport<B, I, E, S>
where
    B: CodecBuilder,
    S: TryStream<Ok = I, Error = E>,
    I: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Item = Result<CodecStream<B::Req, B::Res, B::StreamErr, B::SinkErr>, E>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match ready!(this.inner.try_poll_next(cx)) {
            Some(Ok(stream)) => {
                Poll::Ready(Some(Ok(this.codec_builder.build_codec(Box::new(stream)))))
            }
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}
