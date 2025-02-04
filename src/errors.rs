use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MaybeResyncError<T> {
    #[error("Resync")]
    Resync,
    #[error("Error: {0}")]
    Error(#[from] T),
}

impl From<MaybeResyncError<io::Error>> for MaybeResyncError<ReceiveError> {
    fn from(error: MaybeResyncError<io::Error>) -> Self {
        match error {
            MaybeResyncError::Resync => MaybeResyncError::Resync,
            MaybeResyncError::Error(e) => MaybeResyncError::Error(e.into()),
        }
    }
}

impl From<DecodeError> for MaybeResyncError<ReceiveError> {
    fn from(error: DecodeError) -> Self {
        MaybeResyncError::Error(error.into())
    }
}

#[derive(Debug, Error)]
pub enum ReceiveError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u16),
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),
    #[error("Invalid enum value: {0}")]
    InvalidEnumValue(u8),
}
