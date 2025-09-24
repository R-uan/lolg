#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not bind socket to: {0}")]
    SocketFailed(String),
}
