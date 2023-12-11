use std::time::Duration;

use anyhow::bail;
use bincode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream, time::sleep,
};
use tracing::warn;

pub mod pipe_server;

pub static GATEWAY_PIPE_PATH: &'static str = "/tmp/assyst-gateway-com";

static POLL_FREQUENCY: Duration = Duration::from_secs(10);
static POLL_ATTEMPT_LIMIT: usize = 10;

pub struct Pipe {
    pub stream: UnixStream,
}
impl Pipe {
    pub async fn connect(pipe_location: &str) -> anyhow::Result<Pipe> {
        let stream = UnixStream::connect(pipe_location).await?;
        Ok(Pipe {
            stream,
        })
    }

    pub async fn poll_connect(pipe_location: &str) -> anyhow::Result<Pipe> {
        let mut attempts = 0;

        let pipe: Pipe = loop {
            let pipe = Pipe::connect(pipe_location).await;
            if let Ok(p) = pipe { 
                break p 
            } else if let Err(e) = pipe {
                attempts += 1;
                warn!("{}: connection failed ({}/{}): {}", pipe_location, attempts, POLL_ATTEMPT_LIMIT, e.to_string());
                if attempts >= POLL_ATTEMPT_LIMIT {
                    bail!("timed out waiting for connection");
                }
                sleep(POLL_FREQUENCY).await;
            }
        };

        Ok(pipe)
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