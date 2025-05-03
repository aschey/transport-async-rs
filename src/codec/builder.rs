use std::error::Error;
use std::io;
use std::marker::PhantomData;

use bytes::{Bytes, BytesMut};
use futures_util::{Sink, Stream};

use super::AsyncReadWrite;

pub trait StreamSink<SinkItem>: Stream + Sink<SinkItem> + Unpin + Send {}

impl<T, SinkItem> StreamSink<SinkItem> for T where T: Stream + Sink<SinkItem> + Unpin + Send {}

pub type EncodedStream<Req, Res, StreamErr, SinkErr> =
    Box<dyn StreamSink<Res, Item = Result<Req, StreamErr>, Error = SinkErr>>;

pub trait CodecBuilder: Send {
    type Req: Send;
    type Res: Send;
    type StreamErr: Error + Send + Sync + 'static;
    type SinkErr: Error + Send + Sync + 'static;

    fn build_codec(
        &self,
        incoming: Box<dyn AsyncReadWrite>,
    ) -> EncodedStream<Self::Req, Self::Res, Self::StreamErr, Self::SinkErr>;
}

pub fn codec_builder_fn<F, Req, Res, StreamErr, SinkErr>(
    f: F,
) -> CodecBuilderFn<F, Req, Res, StreamErr, SinkErr>
where
    F: Fn(Box<dyn AsyncReadWrite>) -> EncodedStream<Req, Res, StreamErr, SinkErr>,
{
    CodecBuilderFn {
        f,
        _phantom: Default::default(),
    }
}

pub struct CodecBuilderFn<F, Req, Res, StreamErr, SinkErr>
where
    F: Fn(Box<dyn AsyncReadWrite>) -> EncodedStream<Req, Res, StreamErr, SinkErr>,
{
    f: F,
    _phantom: PhantomData<(Req, Res, StreamErr, SinkErr)>,
}

impl<F, Req, Res, StreamErr, SinkErr> CodecBuilder
    for CodecBuilderFn<F, Req, Res, StreamErr, SinkErr>
where
    F: Fn(Box<dyn AsyncReadWrite>) -> EncodedStream<Req, Res, StreamErr, SinkErr> + Send,
    Req: Send,
    Res: Send,
    StreamErr: Error + Send + Sync + 'static,
    SinkErr: Error + Send + Sync + 'static,
{
    type Req = Req;
    type Res = Res;
    type SinkErr = SinkErr;
    type StreamErr = StreamErr;

    fn build_codec(
        &self,
        incoming: Box<dyn AsyncReadWrite>,
    ) -> EncodedStream<Self::Req, Self::Res, Self::StreamErr, Self::SinkErr> {
        (self.f)(incoming)
    }
}

pub struct LengthDelimitedCodec;

impl LengthDelimitedCodec {
    pub fn client(
        incoming: impl AsyncReadWrite,
    ) -> EncodedStream<BytesMut, Bytes, io::Error, io::Error> {
        Box::new(tokio_util::codec::Framed::new(
            incoming,
            tokio_util::codec::LengthDelimitedCodec::new(),
        ))
    }
}

impl CodecBuilder for LengthDelimitedCodec {
    type Req = BytesMut;
    type Res = Bytes;
    type SinkErr = io::Error;
    type StreamErr = io::Error;

    fn build_codec(
        &self,
        incoming: Box<dyn AsyncReadWrite>,
    ) -> EncodedStream<Self::Req, Self::Res, Self::StreamErr, Self::SinkErr> {
        Self::client(incoming)
    }
}
