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
