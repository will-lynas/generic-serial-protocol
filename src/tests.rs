use super::*;
use std::{os::unix::net::UnixStream, time::Duration};

fn get_test_cases() -> Vec<(Message, Vec<u8>)> {
    vec![
        (
            Message::NoOp(messages::NoOp {}),
            vec![
                0x58, // Start byte
                0x02, 0x00, // Length (2 bytes for message type)
                0x04, 0x00, // Message type (4)
            ],
        ),
        (
            Message::U8(messages::U8 { num: 0x57 }),
            vec![
                0x58, // Start byte
                0x03, 0x00, // Length (2 bytes for message type + 1 byte data)
                0x01, 0x00, // Message type (1)
                0x57, // The u8 value
            ],
        ),
        (
            Message::Bytes(messages::Bytes {
                data: vec![1, 2, 3, 4, 5],
            }),
            vec![
                0x58, // Start byte
                0x07, 0x00, // Length (2 bytes for message type + 5 bytes data)
                0x00, 0x00, // Message type (0)
                1, 2, 3, 4, 5, // The bytes
            ],
        ),
        (
            Message::U16(messages::U16 { num: 0x1234 }),
            vec![
                0x58, // Start byte
                0x04, 0x00, // Length (2 bytes for message type + 2 bytes data)
                0x05, 0x00, // Message type (5)
                0x34, 0x12, // The u16 value in little-endian
            ],
        ),
        (
            Message::Multi(messages::Multi {
                num: 0x42,
                string: "test".to_string(),
            }),
            vec![
                0x58, // Start byte
                0x07,
                0x00, // Length (2 bytes for message type + 1 byte for num + 4 bytes for string)
                0x03, 0x00, // Message type (3)
                0x42, // The u8 value
                b't', b'e', b's', b't', // The string bytes
            ],
        ),
    ]
}

#[test]
fn test_send_raw_bytes() {
    for (message, expected_bytes) in get_test_cases() {
        let (stream1, mut stream2) = UnixStream::pair().unwrap();
        let mut sender = SerialManager::new(stream1);
        sender.send(message).unwrap();

        let mut received_bytes = vec![0u8; expected_bytes.len()];
        stream2.read_exact(&mut received_bytes).unwrap();
        assert_eq!(received_bytes, expected_bytes);

        // Try to read one more byte to verify nothing else was sent
        stream2
            .set_read_timeout(Some(Duration::from_millis(10)))
            .unwrap();
        let mut extra_byte = [0u8; 1];
        assert!(stream2.read_exact(&mut extra_byte).is_err());
    }
}

#[test]
fn test_receive_raw_bytes() {
    for (expected_message, bytes_to_send) in get_test_cases() {
        let (mut stream1, stream2) = UnixStream::pair().unwrap();
        let mut receiver = SerialManager::new(stream2);

        stream1.write_all(&bytes_to_send).unwrap();
        stream1.flush().unwrap();

        let received_message = receiver.receive().unwrap();
        assert_eq!(received_message, expected_message);
    }
}

#[test]
fn test_send_receive() {
    let (stream1, stream2) = UnixStream::pair().unwrap();
    let mut sender = SerialManager::new(stream1);
    let mut receiver = SerialManager::new(stream2);

    let test_messages = vec![
        Message::Bytes(messages::Bytes {
            data: vec![0x48, 0x65, 0x6C, 0x6C, 0x6F],
        }),
        Message::U8(messages::U8 { num: 0x57 }),
        Message::MyString(messages::MyString {
            string: "Hello, world!".to_string(),
        }),
        Message::Multi(messages::Multi {
            num: 0x57,
            string: "Hello, world!".to_string(),
        }),
        Message::NoOp(messages::NoOp {}),
        Message::U16(messages::U16 { num: 0x57 }),
    ];

    for message in test_messages {
        sender.send(message.clone()).unwrap();
        let received = receiver.receive().unwrap();
        assert_eq!(message, received);
    }
}

#[test]
fn test_receive_with_garbage_prefix() {
    let (mut stream1, stream2) = UnixStream::pair().unwrap();
    let mut receiver = SerialManager::new(stream2);

    // Some random bytes that aren't the start byte
    let garbage = vec![0x00, 0xFF, 0x42, 0x13];

    // Take a simple message from our test cases
    let (expected_message, message_bytes) = get_test_cases()[0].clone();

    // Send garbage followed by actual message
    stream1.write_all(&garbage).unwrap();
    stream1.write_all(&message_bytes).unwrap();
    stream1.flush().unwrap();

    // Should still receive correct message
    let received_message = receiver.receive().unwrap();
    assert_eq!(received_message, expected_message);
}

#[test]
fn test_receive_multiple_packets() {
    let (mut stream1, stream2) = UnixStream::pair().unwrap();
    let mut receiver = SerialManager::new(stream2);

    // Take two different messages from our test cases
    let (expected_message1, message_bytes1) = get_test_cases()[0].clone(); // NoOp
    let (expected_message2, message_bytes2) = get_test_cases()[1].clone(); // U8

    // Send both messages in a single write
    let mut combined_bytes = Vec::new();
    combined_bytes.extend(&message_bytes1);
    combined_bytes.extend(&message_bytes2);
    stream1.write_all(&combined_bytes).unwrap();
    stream1.flush().unwrap();

    // Should receive first message
    let received_message1 = receiver.receive().unwrap();
    assert_eq!(received_message1, expected_message1);

    // Should receive second message
    let received_message2 = receiver.receive().unwrap();
    assert_eq!(received_message2, expected_message2);
}

#[test]
fn test_receive_with_interleaved_garbage() {
    let (mut stream1, stream2) = UnixStream::pair().unwrap();
    let mut receiver = SerialManager::new(stream2);

    // Take two different messages from our test cases
    let (expected_message1, message_bytes1) = get_test_cases()[0].clone(); // NoOp
    let (expected_message2, message_bytes2) = get_test_cases()[1].clone(); // U8

    // Some random bytes that aren't the start byte
    let garbage = vec![0x00, 0xFF, 0x42, 0x13, 0x37];

    // Send first message, then garbage, then second message
    let mut combined_bytes = Vec::new();
    combined_bytes.extend(&message_bytes1);
    combined_bytes.extend(&garbage);
    combined_bytes.extend(&message_bytes2);
    stream1.write_all(&combined_bytes).unwrap();
    stream1.flush().unwrap();

    // Should receive first message
    let received_message1 = receiver.receive().unwrap();
    assert_eq!(received_message1, expected_message1);

    // Should receive second message
    let received_message2 = receiver.receive().unwrap();
    assert_eq!(received_message2, expected_message2);
}
