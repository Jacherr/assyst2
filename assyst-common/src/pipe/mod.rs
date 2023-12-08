use bincode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

pub mod pipe_server;

pub struct Pipe {
    pub stream: UnixStream,
}
impl Pipe {
    pub async fn connect(pipe_location: String) -> anyhow::Result<Pipe> {
        let stream = UnixStream::connect(pipe_location.clone()).await?;
        Ok(Pipe {
            stream,
        })
    }

    pub fn new(stream: UnixStream) -> Self {
        Pipe { stream }
    }

    pub async fn read_object<T: DeserializeOwned>(&mut self) -> anyhow::Result<T> {
        let len = self.stream.read_u32().await?;
        let mut data = vec![0u8; len as usize];
        self.stream.read_exact(&mut data).await?;
        Ok(deserialize::<T>(&data)?)
    }

    pub async fn write_object<T: Serialize>(&mut self, obj: T) -> anyhow::Result<()> {
        let buffer = serialize(&obj)?;
        self.stream.write_u32(buffer.len() as u32).await?;
        self.stream.write_all(&buffer).await?;
        Ok(())
    }
}