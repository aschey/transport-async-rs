use std::io;
use std::path::PathBuf;

pub use parity_tokio_ipc::{IntoIpcPath, IpcSecurity, OnConflict, SecurityAttributes, ServerId};
use parity_tokio_ipc::{IpcEndpoint, IpcStream};

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
        let mut endpoint = parity_tokio_ipc::Endpoint::new(params.path, params.on_conflict)?;
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
    type Stream = parity_tokio_ipc::Connection;
    type Params = ConnectionParams;

    async fn connect(params: Self::Params) -> io::Result<Self::Stream> {
        parity_tokio_ipc::Endpoint::connect(params.path).await
    }
}
