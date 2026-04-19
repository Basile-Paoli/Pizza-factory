use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug)]
pub enum FramingError {
    Io(std::io::Error),
    Cbor(String),
}

impl std::fmt::Display for FramingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FramingError::Io(e) => write!(f, "Erreur réseau: {e}"),
            FramingError::Cbor(s) => write!(f, "Erreur CBOR: {s}"),
        }
    }
}

impl std::error::Error for FramingError {}

impl From<std::io::Error> for FramingError {
    fn from(e: std::io::Error) -> Self {
        FramingError::Io(e)
    }
}

pub fn write_message<T: Serialize>(stream: &mut TcpStream, msg: &T) -> Result<(), FramingError> {
    let mut buf = Vec::new();
    ciborium::ser::into_writer(msg, &mut buf)
        .map_err(|e| FramingError::Cbor(e.to_string()))?;

    let len = (buf.len() as u32).to_be_bytes();
    stream.write_all(&len)?;
    stream.write_all(&buf)?;
    stream.flush()?;
    Ok(())
}

pub fn read_message<T: DeserializeOwned>(stream: &mut TcpStream) -> Result<T, FramingError> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload)?;

    ciborium::de::from_reader(&payload[..]).map_err(|e| FramingError::Cbor(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestMsg {
        value: String,
        number: u32,
    }

    fn roundtrip(msg: TestMsg) -> TestMsg {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = thread::spawn(move || {
            let (mut conn, _) = listener.accept().unwrap();
            read_message::<TestMsg>(&mut conn).unwrap()
        });

        let mut client = TcpStream::connect(addr).unwrap();
        write_message(&mut client, &msg).unwrap();

        handle.join().unwrap()
    }

    #[test]
    fn framing_roundtrip() {
        let msg = TestMsg {
            value: "hello".into(),
            number: 42,
        };
        assert_eq!(roundtrip(msg), TestMsg { value: "hello".into(), number: 42 });
    }
}
