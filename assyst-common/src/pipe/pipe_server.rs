use tokio::net::UnixListener;

use super::Pipe;

/// PipeServer is a utility class wrapping [UnixListener] that provides utility functions
/// for listening on a specific file descriptor ("pipe") and accepting a connection from it.
pub struct PipeServer {
    listener: UnixListener,
}
impl PipeServer {
    /// Listen on a specific file descriptor.
    pub fn listen(pipe_location: &str) -> anyhow::Result<PipeServer> {
        let listener = UnixListener::bind(pipe_location)?;
        Ok(PipeServer { listener })
    }

    /// Asynchronously wait for a connection to be recieved from the current listener.
    pub async fn accept_connection(&mut self) -> anyhow::Result<Pipe> {
        let (stream, _) = self.listener.accept().await?;
        let pipe = Pipe::new(stream);
        Ok(pipe)
    }
}
