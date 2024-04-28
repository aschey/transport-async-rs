use std::io;

use bytes::{Bytes, BytesMut};

use crate::AsyncReadWrite;

mod builder;
pub use builder::*;
mod stream;
pub use stream::*;
mod serde;
pub use self::serde::*;
mod serializer;
pub use serializer::*;

#[derive(Clone, Debug, Copy)]
pub enum Codec {
    #[cfg(feature = "bincode")]
    Bincode,
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "messagepack")]
    MessagePack,
    #[cfg(feature = "cbor")]
    Cbor,
}

pub fn length_delimited_codec(
    incoming: impl AsyncReadWrite,
) -> EncodedStream<BytesMut, Bytes, io::Error, io::Error> {
    Box::new(tokio_util::codec::Framed::new(
        incoming,
        tokio_util::codec::LengthDelimitedCodec::new(),
    ))
}
