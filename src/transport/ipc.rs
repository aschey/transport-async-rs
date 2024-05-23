use std::io;
use std::path::PathBuf;

use tipsy::IpcStream;
pub use tipsy::{IntoIpcPath, OnConflict, SecurityAttributes, ServerId};

use super::{Bind, Connect};

pub struct EndpointParams {
    path: PathBuf,
    security_attributes: SecurityAttributes,
    on_conflict: OnConflict,
}

impl EndpointParams {
    pub fn new(
        path: impl IntoIpcPath,
        security_attributes: SecurityAttributes,
        on_conflict: OnConflict,
    ) -> io::Result<Self> {
        Ok(Self {
            path: path.into_ipc_path()?,
            security_attributes,
            on_conflict,
        })
    }
}

pub struct Endpoint {}

impl Bind for Endpoint {
    type Stream = IpcStream;
    type Params = EndpointParams;

    async fn bind(params: Self::Params) -> io::Result<Self::Stream> {
        let mut endpoint = tipsy::Endpoint::new(params.path, params.on_conflict)?;
        endpoint.set_security_attributes(params.security_attributes);
        endpoint.incoming()
    }
}

pub struct Connection {}

pub struct ConnectionParams {
    path: PathBuf,
}

impl ConnectionParams {
    pub fn new(path: impl IntoIpcPath) -> io::Result<Self> {
        Ok(Self {
            path: path.into_ipc_path()?,
        })
    }
}

impl Connect for Connection {
    type Stream = tipsy::Connection;
    type Params = ConnectionParams;

    async fn connect(params: Self::Params) -> io::Result<Self::Stream> {
        tipsy::Endpoint::connect(params.path).await
    }
}
