use anyhow::bail;
use bincode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{net::{UnixListener, UnixStream}, sync::Mutex, io::{AsyncReadExt, AsyncWriteExt}};

pub struct PipeServer {
    listener: UnixListener,
    stream: Option<Mutex<UnixStream>>,
    path: String
}
impl PipeServer {
    pub fn create(pipe_location: String) -> anyhow::Result<PipeServer> {
        let listener = UnixListener::bind(pipe_location.clone())?;
        Ok(PipeServer {
            listener,
            stream: None,
            path: pipe_location
        })
    }

    pub async fn accept_connection(&mut self) -> anyhow::Result<()> {
        let (stream, _) = self.listener.accept().await?;
        self.stream = Some(Mutex::new(stream));
        Ok(())
    }

    pub async fn read_object<T: DeserializeOwned>(&mut self) -> anyhow::Result<T> {
        if let Some(s) = &self.stream {
            let mut lock = s.lock().await;
            let len = lock.read_u32().await?;
            let mut data = vec![0u8; len as usize];
            lock.read_exact(&mut data).await?;
            Ok(deserialize::<T>(&data)?)
        } else {
            bail!("no stream present")
        }
    }

    pub async fn write_object<T: Serialize>(&mut self, obj: T) -> anyhow::Result<()> {
        if let Some(s) = &self.stream {
            let buffer = serialize(&obj)?;
            let mut lock = s.lock().await;
            lock.write_u32(buffer.len() as u32).await?;
            lock.write_all(&buffer).await?;
            Ok(())
        } else {
            bail!("no stream present")
        }
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}