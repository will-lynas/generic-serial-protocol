use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("New message received")]
    NewMessage,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ReceiveError {
    #[error("Read error: {0}")]
    Read(#[from] ReadError),
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u16),
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),
}

impl From<io::Error> for ReceiveError {
    fn from(error: io::Error) -> Self {
        ReceiveError::Read(ReadError::Io(error))
    }
}
