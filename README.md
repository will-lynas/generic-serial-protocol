# Generic Serial Protocol

A Rust implementation of a custom serial protocol for reliable message transmission over serial connections.

## Protocol Specification

### Message Format

```
+------------+------------------+--------------------+------------------+
| Start Byte | Length (2 bytes) | Msg Type (2 bytes) |      Data        |
|    0x58    |     LE u16       |     LE u16         | Variable length  |
+------------+------------------+--------------------+------------------+
```

Every message starts with a start byte (0x58). This is the ground truth for the start of a message, and any other start byte will trigger a resync to the next packet.

To encode a literal 0x58 within the packet (length field, msg type, or data) it must be *escaped*. This is done by replacing the 0x58 with the escape byte (0x42) followed by the original byte XORed with 0x69. This poses an additional problem with sending a literal 0x42. So 0x42 must itself be escaped in the same way.

For example:
- 0x58 becomes [0x42, 0x31] (0x58 ^ 0x69 = 0x31)
- 0x42 becomes [0x42, 0x2B] (0x42 ^ 0x69 = 0x2B)

The length field is the size of the data field plus two bytes for the message type. It is the length *before* escaping, so that the actual number of bytes transmitted may be greater than this number.

All multi-byte fields are transmitted in little-endian format.

## Usage

The protocol can be used with any type that implements `Read + Write`. Here's an example using Unix domain sockets:

```rust
use generic_serial_protocol::{SerialManager, Message, message_types};
use std::os::unix::net::UnixStream;

// Create a pair of connected Unix domain sockets
let (stream1, stream2) = UnixStream::pair().unwrap();

// Create managers for both ends
let mut sender = SerialManager::new(stream1);
let mut receiver = SerialManager::new(stream2);

// Send a message
let message = Message::U8(message_types::U8 { num: 0x57 });
sender.send(message.clone()).unwrap();

// Receive the message
let received = receiver.receive().unwrap();
assert_eq!(message, received);
```