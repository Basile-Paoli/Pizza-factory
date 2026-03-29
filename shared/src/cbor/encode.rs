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
    use crate::cbor::types::{TaggedSocketAddr, TaggedTimestamp, TaggedUuid, TAG_SOCKET_ADDR, TAG_TIMESTAMP, TAG_UUID};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TaggedEnvelope {
        id: TaggedUuid,
        addr: TaggedSocketAddr,
        ts: TaggedTimestamp,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct MixedEnvelope {
        id: TaggedUuid,
        addr: TaggedSocketAddr,
        ts: TaggedTimestamp,
        label: String,
        retry_count: u32,
        ok: bool,
    }

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

    #[test]
    fn serde_roundtrip_struct_with_tagged_fields() {
        let id = TaggedUuid::new(uuid::Uuid::parse_str("299defcb-c217-40e7-9030-af8debf647c6").unwrap());
        let addr = TaggedSocketAddr::new("127.0.0.1:8001".parse().unwrap());
        let ts = TaggedTimestamp::new(1769778234, 895985);
        let original = TaggedEnvelope { id, addr, ts };

        let mut bytes = Vec::new();
        ciborium::into_writer(&original, &mut bytes).unwrap();
        let decoded: TaggedEnvelope = ciborium::from_reader(bytes.as_slice()).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn serde_struct_fields_keep_existing_tag_mapping() {
        let id = TaggedUuid::new(uuid::Uuid::parse_str("299defcb-c217-40e7-9030-af8debf647c6").unwrap());
        let addr = TaggedSocketAddr::new("127.0.0.1:8001".parse().unwrap());
        let ts = TaggedTimestamp::new(1769778234, 895985);
        let env = TaggedEnvelope { id: id.clone(), addr: addr.clone(), ts: ts.clone() };

        let mut bytes = Vec::new();
        ciborium::into_writer(&env, &mut bytes).unwrap();
        let value = crate::cbor::decode::from_bytes(&bytes).unwrap();

        let pairs = match value {
            Value::Map(pairs) => pairs,
            _ => panic!("envelope must encode as CBOR map"),
        };

        let mut seen_id = false;
        let mut seen_addr = false;
        let mut seen_ts = false;

        for (k, v) in pairs {
            let key = match k {
                Value::Text(s) => s,
                _ => continue,
            };

            match key.as_str() {
                "id" => {
                    assert_eq!(decode_uuid(v).unwrap(), id);
                    seen_id = true;
                }
                "addr" => {
                    assert_eq!(decode_socket_addr(v).unwrap(), addr);
                    seen_addr = true;
                }
                "ts" => {
                    assert_eq!(decode_timestamp(v).unwrap(), ts);
                    seen_ts = true;
                }
                _ => {}
            }
        }

        assert!(seen_id && seen_addr && seen_ts);
    }

    #[test]
    fn serde_roundtrip_mixed_struct_with_tagged_and_regular_fields() {
        let original = MixedEnvelope {
            id: TaggedUuid::new(uuid::Uuid::parse_str("299defcb-c217-40e7-9030-af8debf647c6").unwrap()),
            addr: TaggedSocketAddr::new("127.0.0.1:8001".parse().unwrap()),
            ts: TaggedTimestamp::new(1769778234, 895985),
            label: "order-ready".to_string(),
            retry_count: 3,
            ok: true,
        };

        let mut bytes = Vec::new();
        ciborium::into_writer(&original, &mut bytes).unwrap();
        let decoded: MixedEnvelope = ciborium::from_reader(bytes.as_slice()).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn serde_mixed_struct_all_fields_preserved() {
        let id = TaggedUuid::new(uuid::Uuid::parse_str("aaaabbbb-cccc-dddd-eeee-ffff00001111").unwrap());
        let addr = TaggedSocketAddr::new("192.168.1.100:9999".parse().unwrap());
        let ts = TaggedTimestamp::new(1000000000, 500000);
        let original = MixedEnvelope {
            id: id.clone(),
            addr: addr.clone(),
            ts: ts.clone(),
            label: "test-label".to_string(),
            retry_count: 42,
            ok: false,
        };

        let mut bytes = Vec::new();
        ciborium::into_writer(&original, &mut bytes).unwrap();
        let decoded: MixedEnvelope = ciborium::from_reader(bytes.as_slice()).unwrap();

        // Vérifier chaque champ individuellement
        assert_eq!(decoded.id, id);
        assert_eq!(decoded.addr, addr);
        assert_eq!(decoded.ts, ts);
        assert_eq!(decoded.label, "test-label".to_string());
        assert_eq!(decoded.retry_count, 42);
        assert_eq!(decoded.ok, false);
    }

    #[test]
    fn serde_mixed_struct_cbor_tags_in_raw_value() {
        let original = MixedEnvelope {
            id: TaggedUuid::new(uuid::Uuid::parse_str("11223344-5566-7788-9900-aabbccddeeff").unwrap()),
            addr: TaggedSocketAddr::new("10.0.0.1:1234".parse().unwrap()),
            ts: TaggedTimestamp::new(1500000000, 250000),
            label: "network-test".to_string(),
            retry_count: 5,
            ok: true,
        };

        // Sérialiser
        let mut bytes = Vec::new();
        ciborium::into_writer(&original, &mut bytes).unwrap();

        // Décoder comme Value brut pour inspecter les tags
        let raw_value = crate::cbor::decode::from_bytes(&bytes).unwrap();

        if let Value::Map(pairs) = raw_value {
            let mut has_id_tag = false;
            let mut has_addr_tag = false;
            let mut has_ts_tag = false;

            for (k, v) in &pairs {
                if let Value::Text(key) = k {
                    match key.as_str() {
                        "id" => {
                            // Vérifier que la valeur est bien un Tag 37
                            if let Value::Tag(tag, _) = v {
                                assert_eq!(*tag, TAG_UUID);
                                has_id_tag = true;
                            }
                        }
                        "addr" => {
                            if let Value::Tag(tag, _) = v {
                                assert_eq!(*tag, TAG_SOCKET_ADDR);
                                has_addr_tag = true;
                            }
                        }
                        "ts" => {
                            if let Value::Tag(tag, _) = v {
                                assert_eq!(*tag, TAG_TIMESTAMP);
                                has_ts_tag = true;
                            }
                        }
                        _ => {}
                    }
                }
            }

            assert!(has_id_tag, "ID tag was not preserved");
            assert!(has_addr_tag, "SocketAddr tag was not preserved");
            assert!(has_ts_tag, "Timestamp tag was not preserved");
        } else {
            panic!("Expected Value::Map, got something else");
        }
    }

    #[test]
    fn serde_mixed_struct_with_empty_string() {
        let original = MixedEnvelope {
            id: TaggedUuid::new(uuid::Uuid::new_v4()),
            addr: TaggedSocketAddr::new("127.0.0.1:0".parse().unwrap()),
            ts: TaggedTimestamp::new(0, 0),
            label: "".to_string(),
            retry_count: 0,
            ok: false,
        };

        let mut bytes = Vec::new();
        ciborium::into_writer(&original, &mut bytes).unwrap();
        let decoded: MixedEnvelope = ciborium::from_reader(bytes.as_slice()).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn serde_mixed_struct_max_values() {
        let original = MixedEnvelope {
            id: TaggedUuid::new(uuid::Uuid::new_v4()),
            addr: TaggedSocketAddr::new("255.255.255.255:65535".parse().unwrap()),
            ts: TaggedTimestamp::new(i64::MAX, i64::MAX),
            label: "a".repeat(1000),
            retry_count: u32::MAX,
            ok: true,
        };

        let mut bytes = Vec::new();
        ciborium::into_writer(&original, &mut bytes).unwrap();
        let decoded: MixedEnvelope = ciborium::from_reader(bytes.as_slice()).unwrap();

        assert_eq!(original.id, decoded.id);
        assert_eq!(original.addr, decoded.addr);
        assert_eq!(original.ts, decoded.ts);
        assert_eq!(original.label, decoded.label);
        assert_eq!(original.retry_count, decoded.retry_count);
        assert_eq!(original.ok, decoded.ok);
    }
}
