use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const MAGIC: [u8; 4] = *b"DCCH";
pub const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Handshake {
    pub magic: [u8; 4],
    pub version: u32,
    pub capabilities: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum HandshakeError {
    #[error("Invalid magic number")]
    InvalidMagic,
    #[error("Unsupported version: expected {0}, got {1}")]
    UnsupportedVersion(u32, u32),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Handshake {
    pub fn new() -> Self {
        Self {
            magic: MAGIC,
            version: CURRENT_VERSION,
            capabilities: vec!["grpc".to_string()],
        }
    }

    pub async fn send<S>(&self, stream: &mut S) -> Result<(), HandshakeError>
    where
        S: AsyncWrite + Unpin,
    {
        let json = serde_json::to_vec(self)?;
        let len = json.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&json).await?;
        stream.flush().await?;
        Ok(())
    }

    pub async fn receive<S>(stream: &mut S) -> Result<Self, HandshakeError>
    where
        S: AsyncRead + Unpin,
    {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf);
        
        let mut buf = vec![0u8; len as usize];
        stream.read_exact(&mut buf).await?;
        
        let handshake: Self = serde_json::from_slice(&buf)?;
        handshake.validate()?;
        Ok(handshake)
    }

    pub fn validate(&self) -> Result<(), HandshakeError> {
        if self.magic != MAGIC {
            return Err(HandshakeError::InvalidMagic);
        }
        if self.version != CURRENT_VERSION {
            return Err(HandshakeError::UnsupportedVersion(CURRENT_VERSION, self.version));
        }
        Ok(())
    }
}
