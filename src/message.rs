use crate::messages;

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Bytes(messages::Bytes),
    U8(messages::U8),
    MyString(messages::MyString),
    Multi(messages::Multi),
    NoOp(messages::NoOp),
    U16(messages::U16),
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

    pub fn from_bytes(message_type: u16, data: Vec<u8>) -> Self {
        match message_type {
            0 => Message::Bytes(messages::Bytes { data }),
            1 => Message::U8(messages::U8 { num: data[0] }),
            2 => Message::MyString(messages::MyString {
                string: String::from_utf8(data).unwrap(),
            }),
            3 => Message::Multi(messages::Multi {
                num: data[0],
                string: String::from_utf8(data[1..].to_vec()).unwrap(),
            }),
            4 => Message::NoOp(messages::NoOp {}),
            5 => Message::U16(messages::U16 {
                num: u16::from_le_bytes([data[0], data[1]]),
            }),
            _ => panic!("Invalid message type: {}", message_type),
        }
    }
}
