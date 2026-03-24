use ciborium::Value;

use crate::cbor::error::CborError;
use crate::cbor::types::{
    TaggedSocketAddr, TaggedTimestamp, TaggedUuid, TAG_SOCKET_ADDR, TAG_TIMESTAMP, TAG_UUID,
};
use crate::cbor_encode_tagged;


pub fn encode_uuid(tagged: &TaggedUuid) -> Value {
    let bytes = tagged.inner().as_bytes().to_vec();
    cbor_encode_tagged!(TAG_UUID, Value::Bytes(bytes))
}


pub fn encode_socket_addr(tagged: &TaggedSocketAddr) -> Value {
    cbor_encode_tagged!(TAG_SOCKET_ADDR, Value::Text(tagged.inner().to_string()))
}

pub fn encode_timestamp(ts: &TaggedTimestamp) -> Value {
    let pairs = vec![
        (
            Value::Integer(1_i64.into()),
            Value::Integer(ts.seconds.into()),
        ),
        (
            Value::Integer(ciborium::value::Integer::try_from(-6_i64).expect("constante valide"),),
            Value::Integer(ts.microseconds.into()),
        ),
    ];
    cbor_encode_tagged!(TAG_TIMESTAMP, Value::Map(pairs))
}

pub fn to_bytes(value: &Value) -> Result<Vec<u8>, CborError> {
    let mut buf = Vec::new();
    ciborium::into_writer(value, &mut buf)
        .map_err(|e| CborError::EncodeError(e.to_string()))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbor::decode::{decode_socket_addr, decode_timestamp, decode_uuid};
    use crate::cbor::types::{TaggedSocketAddr, TaggedTimestamp, TaggedUuid};

    #[test]
    fn roundtrip_uuid() {
        let raw = uuid::Uuid::parse_str("299defcb-c217-40e7-9030-af8debf647c6").unwrap();
        let original = TaggedUuid::new(raw);
        let encoded = encode_uuid(&original);
        let decoded = decode_uuid(encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_socket_addr() {
        let addr: std::net::SocketAddr = "127.0.0.1:8001".parse().unwrap();
        let original = TaggedSocketAddr::new(addr);
        let encoded = encode_socket_addr(&original);
        let decoded = decode_socket_addr(encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_timestamp() {
        let original = TaggedTimestamp::new(1769778234, 895985);
        let encoded = encode_timestamp(&original);
        let decoded = decode_timestamp(encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_bytes_uuid() {
        let raw = uuid::Uuid::new_v4();
        let original = TaggedUuid::new(raw);
        let cbor_val = encode_uuid(&original);
        let bytes = to_bytes(&cbor_val).unwrap();
        let val2 = crate::cbor::decode::from_bytes(&bytes).unwrap();
        let decoded = decode_uuid(val2).unwrap();
        assert_eq!(original, decoded);
    }
}
