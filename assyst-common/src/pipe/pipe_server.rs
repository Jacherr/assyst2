use tokio::{net::UnixListener, sync::Mutex};

use super::Pipe;

pub struct PipeServer {
    listener: UnixListener,
    pub pipe: Option<Mutex<Pipe>>,
}
impl PipeServer {
    pub fn listen(pipe_location: String) -> anyhow::Result<PipeServer> {
        let listener = UnixListener::bind(pipe_location.clone())?;
        Ok(PipeServer {
            listener,
            pipe: None
        })
    }

    pub async fn accept_connection(&mut self) -> anyhow::Result<()> {
        let (stream, _) = self.listener.accept().await?;
        self.pipe = Some(Mutex::new(Pipe::new(stream)));
        Ok(())
    }
}