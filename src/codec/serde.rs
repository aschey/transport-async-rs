use std::io;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::LengthDelimitedCodec;

use super::serializer::CodecSerializer;
use super::EncodedStream;
use super::{AsyncReadWrite, Codec, CodecBuilder};

pub fn serde_codec<Req, Res>(
    incoming: impl AsyncRead + AsyncWrite + Send + Unpin + 'static,
    codec: Codec,
) -> EncodedStream<Req, Res, io::Error, io::Error>
where
    Req: Serialize + for<'de> Deserialize<'de> + Unpin + Send + 'static,
    Res: Serialize + for<'de> Deserialize<'de> + Unpin + Send + 'static,
{
    let stream = tokio_util::codec::Framed::new(incoming, LengthDelimitedCodec::new());

    let stream = tokio_serde::Framed::new(stream, CodecSerializer::new(codec));
    Box::new(stream)
}

#[derive(Clone, Debug)]
pub struct SerdeCodec<Req, Res> {
    _phantom: PhantomData<(Req, Res)>,
    codec: super::Codec,
}

impl<Req, Res> SerdeCodec<Req, Res> {
    pub fn new(codec: super::Codec) -> Self {
        Self {
            codec,
            _phantom: Default::default(),
        }
    }

    pub fn client(
        &self,
        incoming: impl AsyncRead + AsyncWrite + Send + Unpin + 'static,
    ) -> EncodedStream<Res, Req, io::Error, io::Error>
    where
        Req: Serialize + for<'de> Deserialize<'de> + Unpin + Send + 'static,
        Res: Serialize + for<'de> Deserialize<'de> + Unpin + Send + 'static,
    {
        serde_codec(incoming, self.codec)
    }
}

impl<Req, Res> CodecBuilder for SerdeCodec<Req, Res>
where
    Req: Serialize + for<'de> Deserialize<'de> + Unpin + Send + 'static,
    Res: Serialize + for<'de> Deserialize<'de> + Unpin + Send + 'static,
{
    type Req = Req;
    type Res = Res;
    type SinkErr = io::Error;
    type StreamErr = io::Error;

    fn build_codec(
        &self,
        incoming: Box<dyn AsyncReadWrite>,
    ) -> EncodedStream<Self::Req, Self::Res, Self::StreamErr, Self::SinkErr> {
        serde_codec(incoming, self.codec)
    }
}
