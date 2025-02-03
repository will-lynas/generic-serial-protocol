use super::*;
use std::{os::unix::net::UnixStream, time::Duration};

fn get_test_cases() -> Vec<(Message, Vec<u8>)> {
    vec![
        (
            Message::NoOp(message_types::NoOp {}),
            vec![
                START_BYTE, // Start byte
                0x02, 0x00, // Length (2 bytes for message type)
                0x04, 0x00, // Message type (4)
            ],
        ),
        (
            Message::U8(message_types::U8 { num: 0x57 }),
            vec![
                START_BYTE, // Start byte
                0x03, 0x00, // Length (2 bytes for message type + 1 byte data)
                0x01, 0x00, // Message type (1)
                0x57, // The u8 value
            ],
        ),
        (
            Message::Bytes(message_types::Bytes {
                data: vec![1, 2, 3, 4, 5],
            }),
            vec![
                START_BYTE, // Start byte
                0x07, 0x00, // Length (2 bytes for message type + 5 bytes data)
                0x00, 0x00, // Message type (0)
                1, 2, 3, 4, 5, // The bytes
            ],
        ),
        (
            Message::U16(message_types::U16 { num: 0x1234 }),
            vec![
                START_BYTE, // Start byte
                0x04, 0x00, // Length (2 bytes for message type + 2 bytes data)
                0x05, 0x00, // Message type (5)
                0x34, 0x12, // The u16 value in little-endian
            ],
        ),
        (
            Message::Multi(message_types::Multi {
                num: 0x41,
                string: "test".to_string(),
            }),
            vec![
                START_BYTE, // Start byte
                0x07,
                0x00, // Length (2 bytes for message type + 1 byte for num + 4 bytes for string)
                0x03, 0x00, // Message type (3)
                0x41, // The u8 value
                b't', b'e', b's', b't', // The string bytes
            ],
        ),
        // Test case with START_BYTE in data
        (
            Message::Bytes(message_types::Bytes {
                data: vec![START_BYTE],
            }),
            vec![
                START_BYTE, // Start byte
                0x03,
                0x00, // Length (2 bytes for message type + 1 byte data)
                0x00,
                0x00, // Message type (0)
                ESCAPE_BYTE,
                START_BYTE ^ XOR_BYTE, // Escaped START_BYTE
            ],
        ),
        // Test case with ESCAPE_BYTE in data
        (
            Message::Bytes(message_types::Bytes {
                data: vec![ESCAPE_BYTE],
            }),
            vec![
                START_BYTE, // Start byte
                0x03,
                0x00, // Length (2 bytes for message type + 1 byte data)
                0x00,
                0x00, // Message type (0)
                ESCAPE_BYTE,
                ESCAPE_BYTE ^ XOR_BYTE, // Escaped ESCAPE_BYTE
            ],
        ),
        // Test case with multiple bytes needing escaping
        (
            Message::Bytes(message_types::Bytes {
                data: vec![START_BYTE, ESCAPE_BYTE, START_BYTE],
            }),
            vec![
                START_BYTE, // Start byte
                0x05,
                0x00, // Length (2 bytes for message type + 3 bytes data)
                0x00,
                0x00, // Message type (0)
                // Each byte that needs escaping is preceded by ESCAPE_BYTE and XORed with XOR_BYTE
                ESCAPE_BYTE,
                START_BYTE ^ XOR_BYTE,
                ESCAPE_BYTE,
                ESCAPE_BYTE ^ XOR_BYTE,
                ESCAPE_BYTE,
                START_BYTE ^ XOR_BYTE,
            ],
        ),
        // Test case with ESCAPE_BYTE in length field (length = 0x0042)
        (
            Message::Bytes(message_types::Bytes {
                data: vec![0; 0x40], // 64 bytes of data + 2 bytes message type = 0x42 (ESCAPE_BYTE)
            }),
            {
                let mut bytes = vec![
                    START_BYTE,  // Start byte
                    ESCAPE_BYTE, // Escape the 0x42 in length
                    ESCAPE_BYTE ^ XOR_BYTE,
                    0x00,
                    0x00,
                    0x00, // Message type (0)
                ];
                bytes.extend(vec![0; 0x40]); // Add 64 zeros
                bytes
            },
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

    // Get just the messages from test cases, discarding the raw bytes
    let test_messages: Vec<Message> = get_test_cases().into_iter().map(|(msg, _)| msg).collect();

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

#[test]
fn test_receive_interrupted_message() {
    let (mut stream1, stream2) = UnixStream::pair().unwrap();
    let mut receiver = SerialManager::new(stream2);

    // Take a message from our test cases that will be interrupted
    let (_, message_bytes1) = get_test_cases()[1].clone(); // U8 message

    // Take another message that will be received successfully
    let (expected_message2, message_bytes2) = get_test_cases()[0].clone(); // NoOp message

    // Send first half of first message
    let half_length = message_bytes1.len() / 2;
    stream1.write_all(&message_bytes1[..half_length]).unwrap();

    // Send the complete second message
    stream1.write_all(&message_bytes2).unwrap();
    stream1.flush().unwrap();

    // Should receive the second message correctly, first message should be discarded
    let received_message = receiver.receive().unwrap();
    assert_eq!(received_message, expected_message2);
}
