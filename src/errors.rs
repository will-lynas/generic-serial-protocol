use std::io;

#[derive(Debug)]
pub enum ReadError {
    NewMessage,
    Io(io::Error),
}

#[derive(Debug)]
pub enum ReceiveError {
    Read(ReadError),
    Decode(DecodeError),
}

#[derive(Debug)]
pub enum DecodeError {
    InvalidMessageType(u16),
    InvalidUtf8(std::string::FromUtf8Error),
}

impl From<io::Error> for ReadError {
    fn from(error: io::Error) -> Self {
        ReadError::Io(error)
    }
}

impl From<io::Error> for ReceiveError {
    fn from(error: io::Error) -> Self {
        ReceiveError::Read(ReadError::Io(error))
    }
}

impl From<ReadError> for ReceiveError {
    fn from(error: ReadError) -> Self {
        ReceiveError::Read(error)
    }
}

impl From<DecodeError> for ReceiveError {
    fn from(error: DecodeError) -> Self {
        ReceiveError::Decode(error)
    }
}

impl From<std::string::FromUtf8Error> for DecodeError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        DecodeError::InvalidUtf8(err)
    }
}
