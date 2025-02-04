#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]

mod errors;
mod message;
mod message_types;
mod serial_manager;

pub use errors::{DecodeError, ReadError, ReceiveError};
pub use message::Message;
pub use serial_manager::SerialManager;
