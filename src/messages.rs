#[derive(Debug, PartialEq, Clone)]
pub struct Bytes {
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct U8 {
    pub num: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct U16 {
    pub num: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MyString {
    pub string: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Multi {
    pub num: u8,
    pub string: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NoOp {}
