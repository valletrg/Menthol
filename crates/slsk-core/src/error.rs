use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("protocol error: {0}")]
    Proto(#[from] slsk_proto::error::ProtoError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("tokio error: {0}")]
    Tokio(#[from] tokio::task::JoinError),
}
