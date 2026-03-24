use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct Version {
    pub(super) counter: u64,
    pub(super) generation: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality() {
        let v1 = Version { counter: 1, generation: 100 };
        let v2 = Version { counter: 1, generation: 100 };
        let v3 = Version { counter: 2, generation: 100 };
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn copy_semantics() {
        let v = Version { counter: 5, generation: 10 };
        let v2 = v; // Copy — original must still be usable
        assert_eq!(v, v2);
    }

    #[test]
    fn serialize_round_trip() {
        let v = Version { counter: 42, generation: 99 };
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&v, &mut buf).unwrap();
        let v2: Version = ciborium::de::from_reader(&buf[..]).unwrap();
        assert_eq!(v, v2);
    }
}
