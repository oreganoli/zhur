use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::errors::IpcError;

pub struct UnixServer {
    /// Internal message buffer.
    buf: Vec<u8>,
    /// Underlying Unix socket stream.
    stream: UnixStream,
}

impl UnixServer {
    pub fn new(buffer_size: usize, stream: UnixStream) -> Self {
        Self {
            buf: vec![0; buffer_size],
            stream,
        }
    }
    pub async fn get_request<Req: DeserializeOwned>(&mut self) -> Result<Req, IpcError> {
        match self.stream.readable().await {
            Ok(_) => (),
            Err(e) => return Err(e.into()),
        };
        let len = match self.stream.read(&mut self.buf).await {
            Ok(0) => return Err(IpcError::ClientDisconnected),
            Ok(l) => l,
            Err(e) => return Err(e.into()),
        };
        let result = match bincode::deserialize::<Req>(&self.buf[0..len]) {
            Ok(r) => Ok(r),
            Err(_) => Err(IpcError::RequestDeserialization),
        };
        self.buf.clear();
        result
    }
    pub async fn send_response<Res: Serialize>(&mut self, response: &Res) -> Result<(), IpcError> {
        match self.stream.writable().await {
            Ok(_) => (),
            Err(e) => return Err(e.into()),
        };
        let response_bytes = match bincode::serialize(response) {
            Ok(v) => v,
            Err(_) => return Err(IpcError::ResponseSerialization),
        };
        match self.stream.write(&response_bytes).await {
            Ok(0) => Err(IpcError::ClientDisconnected),
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
