use std::io::{self, Read, Write};

pub struct SerialManager<T>
where
    T: Read + Write,
{
    connection: T,
}

impl<T> SerialManager<T>
where
    T: Read + Write,
{
    pub fn new(connection: T) -> Self {
        Self { connection }
    }

    pub fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.connection.write_all(data)?;
        self.connection.flush()?;
        Ok(())
    }

    pub fn receive(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; 1024];
        let n = self.connection.read(&mut buffer)?;
        buffer.truncate(n);
        Ok(buffer)
    }
}

fn main() {
    // Main function left empty for now
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    #[test]
    fn test_serial_communication() {
        let (stream1, stream2) = UnixStream::pair().unwrap();

        let mut sender = SerialManager::new(stream1);
        let mut receiver = SerialManager::new(stream2);

        let data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F];
        sender.send(&data).unwrap();
        let received = receiver.receive().unwrap();
        assert_eq!(data, received);
    }
}
