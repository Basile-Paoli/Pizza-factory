#[macro_export]
macro_rules! cbor_encode_tagged {
    ($tag:expr, $value:expr) => {
        ciborium::Value::Tag($tag, Box::new($value))
    };
}

#[macro_export]
macro_rules! cbor_decode_check_tag {
    ($value:expr, $expected_tag:expr) => {{
        use $crate::cbor::error::CborError;
        match $value {
            ciborium::Value::Tag(got_tag, inner) => {
                if got_tag != $expected_tag {
                    Err(CborError::WrongTag {
                        expected: $expected_tag,
                        got: got_tag,
                    })
                } else {
                    Ok(*inner)
                }
            }
            _ => Err(CborError::NotTagged),
        }
    }};
}


#[macro_export]
macro_rules! cbor_map_get_int {
    ($pairs:expr, $key:expr) => {{
        use $crate::cbor::error::CborError;
        let key_i64: i64 = $key;
        $pairs
            .iter()
            .find_map(|(k, v)| {
                if let ciborium::Value::Integer(n) = k {
                    let n_i64: i64 = i64::try_from(*n).ok()?;
                    if n_i64 == key_i64 {
                        return Some(v.clone());
                    }
                }
                None
            })
            .ok_or(CborError::MissingKey { key: key_i64 })
    }};
}
