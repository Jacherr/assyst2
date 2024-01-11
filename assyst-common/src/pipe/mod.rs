use std::time::Duration;

use anyhow::bail;
use bincode::{deserialize, serialize};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::sleep;
use tracing::warn;

pub mod pipe_server;

pub static GATEWAY_PIPE_PATH: &str = "/tmp/assyst-gateway-com";

static POLL_FREQUENCY: Duration = Duration::from_secs(10);
static POLL_ATTEMPT_LIMIT: usize = 10;

/// Pipe is a utility class that wraps a [UnixStream], providing helper functions for easy reading
/// and writing of serde-Serializable types via Bincode.
pub struct Pipe {
    pub stream: UnixStream,
}
impl Pipe {
    /// Connect to a specific file descriptor.
    pub async fn connect(pipe_location: &str) -> anyhow::Result<Pipe> {
        let stream = UnixStream::connect(pipe_location).await?;
        Ok(Pipe { stream })
    }

    /// Repeatedly attempt to connect to a specific file descriptor, until a maximum retry threshold
    /// is reached.
    pub async fn poll_connect(pipe_location: &str) -> anyhow::Result<Pipe> {
        let mut attempts = 0;

        let pipe: Pipe = loop {
            let pipe = Pipe::connect(pipe_location).await;
            if let Ok(p) = pipe {
                break p;
            } else if let Err(e) = pipe {
                attempts += 1;
                warn!(
                    "{}: connection failed ({}/{}): {}",
                    pipe_location,
                    attempts,
                    POLL_ATTEMPT_LIMIT,
                    e.to_string()
                );
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

    /// Read a Bincode-deserializable object from this stream.
    ///
    /// This function will return an Err if the stream is prematurely closed, or if Bincode is not
    /// able to deserialize the data to the specified type.
    pub async fn read_object<T: DeserializeOwned>(&mut self) -> anyhow::Result<T> {
        let len = self.stream.read_u32().await?;
        let mut data = vec![0u8; len as usize];
        self.stream.read_exact(&mut data).await?;
        Ok(deserialize::<T>(&data)?)
    }

    /// Write a Bincode-serializable object to this stream.
    ///
    /// This function will return an Err if the stream is prematurely closed, or if Bincode is not
    /// able to serialize the data to the specified type.
    pub async fn write_object<T: Serialize>(&mut self, obj: T) -> anyhow::Result<()> {
        let buffer = serialize(&obj)?;
        self.stream.write_u32(buffer.len() as u32).await?;
        self.stream.write_all(&buffer).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
