use crate::message_types;

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Bytes(message_types::Bytes),
    U8(message_types::U8),
    MyString(message_types::MyString),
    Multi(message_types::Multi),
    NoOp(message_types::NoOp),
    U16(message_types::U16),
}

#[derive(Debug)]
pub enum DecodeError {
    InvalidMessageType(u16),
    InvalidUtf8(std::string::FromUtf8Error),
}

impl From<std::string::FromUtf8Error> for DecodeError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        DecodeError::InvalidUtf8(err)
    }
}

impl Message {
    pub fn message_type(&self) -> u16 {
        match self {
            Message::Bytes(_) => 0,
            Message::U8(_) => 1,
            Message::MyString(_) => 2,
            Message::Multi(_) => 3,
            Message::NoOp(_) => 4,
            Message::U16(_) => 5,
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self {
            Message::Bytes(msg) => bytes.extend(msg.data),
            Message::U8(msg) => bytes.push(msg.num),
            Message::MyString(msg) => bytes.extend(msg.string.as_bytes()),
            Message::Multi(msg) => {
                bytes.push(msg.num);
                bytes.extend(msg.string.as_bytes());
            }
            Message::NoOp(_) => {}
            Message::U16(msg) => bytes.extend(msg.num.to_le_bytes()),
        }

        bytes
    }

    pub fn from_bytes(message_type: u16, data: Vec<u8>) -> Result<Self, DecodeError> {
        Ok(match message_type {
            0 => Message::Bytes(message_types::Bytes { data }),
            1 => Message::U8(message_types::U8 { num: data[0] }),
            2 => Message::MyString(message_types::MyString {
                string: String::from_utf8(data)?,
            }),
            3 => Message::Multi(message_types::Multi {
                num: data[0],
                string: String::from_utf8(data[1..].to_vec())?,
            }),
            4 => Message::NoOp(message_types::NoOp {}),
            5 => Message::U16(message_types::U16 {
                num: u16::from_le_bytes([data[0], data[1]]),
            }),
            _ => return Err(DecodeError::InvalidMessageType(message_type)),
        })
    }
}
