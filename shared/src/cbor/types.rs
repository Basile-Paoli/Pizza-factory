use std::net::SocketAddr;
use ciborium::Value;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

pub const TAG_UUID: u64 = 37;
pub const TAG_SOCKET_ADDR: u64 = 260;
pub const TAG_TIMESTAMP: u64 = 1001;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaggedUuid(pub Uuid);

impl TaggedUuid {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn inner(&self) -> &Uuid {
        &self.0
    }

    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for TaggedUuid {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl From<TaggedUuid> for Uuid {
    fn from(t: TaggedUuid) -> Self {
        t.0
    }
}

impl Serialize for TaggedUuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = crate::cbor::encode::encode_uuid(self);
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TaggedUuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        crate::cbor::decode::decode_uuid(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TaggedSocketAddr(pub SocketAddr);

impl TaggedSocketAddr {
    pub fn new(addr: SocketAddr) -> Self {
        Self(addr)
    }

    pub fn inner(&self) -> &SocketAddr {
        &self.0
    }

    pub fn into_inner(self) -> SocketAddr {
        self.0
    }
}

impl From<SocketAddr> for TaggedSocketAddr {
    fn from(a: SocketAddr) -> Self {
        Self(a)
    }
}

impl From<TaggedSocketAddr> for SocketAddr {
    fn from(t: TaggedSocketAddr) -> Self {
        t.0
    }
}

impl Serialize for TaggedSocketAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = crate::cbor::encode::encode_socket_addr(self);
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TaggedSocketAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        crate::cbor::decode::decode_socket_addr(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedTimestamp {
    pub seconds: i64,
    pub microseconds: i64,
}

impl TaggedTimestamp {
    pub fn new(seconds: i64, microseconds: i64) -> Self {
        Self { seconds, microseconds }
    }

    pub fn to_micros(&self) -> i64 {
        self.seconds * 1_000_000 + self.microseconds
    }

    pub fn from_micros(us: i64) -> Self {
        Self {
            seconds: us / 1_000_000,
            microseconds: us % 1_000_000,
        }
    }
}

impl Serialize for TaggedTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = crate::cbor::encode::encode_timestamp(self);
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TaggedTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        crate::cbor::decode::decode_timestamp(value).map_err(serde::de::Error::custom)
    }
}
