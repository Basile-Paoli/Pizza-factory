use std::net::SocketAddr;

use ciborium::Value;
use uuid::Uuid;

use crate::cbor::error::CborError;
use crate::cbor::types::{
    TaggedSocketAddr, TaggedTimestamp, TaggedUuid, TAG_SOCKET_ADDR, TAG_TIMESTAMP, TAG_UUID,
};
use crate::{cbor_decode_check_tag, cbor_map_get_int};

pub fn decode_uuid(value: Value) -> Result<TaggedUuid, CborError> {
    let inner = cbor_decode_check_tag!(value, TAG_UUID)?;

    match inner {
        Value::Text(text) => {
            Ok(TaggedUuid::new(Uuid::parse_str(&text).map_err(|e| CborError::InvalidValue {
                context: "UUID : parsing de la string a échoué :",
            })?))
        }
        _ => Err(CborError::InvalidValue {
            context: "UUID : valeur interne doit être bstr",
        }),
    }
}

pub fn decode_socket_addr(value: Value) -> Result<TaggedSocketAddr, CborError> {
    let inner = cbor_decode_check_tag!(value, TAG_SOCKET_ADDR)?;

    match inner {
        Value::Text(s) => {
            let addr: SocketAddr = s.parse()?;
            Ok(TaggedSocketAddr::new(addr))
        }
        _ => Err(CborError::InvalidValue {
            context: "SocketAddr : valeur interne doit être tstr",
        }),
    }
}

pub fn decode_timestamp(value: Value) -> Result<TaggedTimestamp, CborError> {
    let inner = cbor_decode_check_tag!(value, TAG_TIMESTAMP)?;

    match inner {
        Value::Map(pairs) => {
            // Clé 1 → secondes
            let sec_val = cbor_map_get_int!(pairs, 1_i64)?;
            let seconds = extract_integer(sec_val, "timestamp.seconds")?;

            // Clé -6 → microsecondes
            let us_val = cbor_map_get_int!(pairs, -6_i64)?;
            let microseconds = extract_integer(us_val, "timestamp.microseconds")?;

            Ok(TaggedTimestamp::new(seconds, microseconds))
        }
        _ => Err(CborError::InvalidValue {
            context: "Timestamp : valeur interne doit être une map CBOR",
        }),
    }
}

fn extract_integer(value: Value, context: &'static str) -> Result<i64, CborError> {
    match value {
        Value::Integer(n) => i64::try_from(n).map_err(|_| CborError::InvalidValue { context }),
        _ => Err(CborError::InvalidValue { context }),
    }
}

pub fn from_bytes(bytes: &[u8]) -> Result<Value, CborError> {
    ciborium::from_reader(bytes).map_err(|e| CborError::DecodeError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciborium::Value;

    fn make_timestamp_value(sec: i64, us: i64) -> Value {
        Value::Tag(
            TAG_TIMESTAMP,
            Box::new(Value::Map(vec![
                (Value::Integer(1.into()), Value::Integer(sec.into())),
                (
                    Value::Integer(ciborium::value::Integer::try_from(-6_i64).unwrap()),
                    Value::Integer(us.into()),
                ),
            ])),
        )
    }

    #[test]
    fn decode_timestamp_ok() {
        let v = make_timestamp_value(1769778234, 895985);
        let ts = decode_timestamp(v).unwrap();
        assert_eq!(ts.seconds, 1769778234);
        assert_eq!(ts.microseconds, 895985);
    }

    #[test]
    fn decode_timestamp_wrong_tag() {
        let v = Value::Tag(42, Box::new(Value::Map(vec![])));
        let err = decode_timestamp(v).unwrap_err();
        assert!(matches!(err, CborError::WrongTag { expected: 1001, got: 42 }));
    }

    #[test]
    fn decode_socket_addr_ok() {
        let v = Value::Tag(TAG_SOCKET_ADDR, Box::new(Value::Text("127.0.0.1:8001".into())));
        let addr = decode_socket_addr(v).unwrap();
        assert_eq!(addr.inner().to_string(), "127.0.0.1:8001");
    }

    #[test]
    fn decode_uuid_ok() {
        let raw = uuid::Uuid::parse_str("299defcb-c217-40e7-9030-af8debf647c6").unwrap();
        let v = Value::Tag(TAG_UUID, Box::new(Value::Text(raw.to_string())));
        let tagged = decode_uuid(v).unwrap();
        assert_eq!(tagged.inner(), &raw);
    }
}
