use ciborium::tag::Required;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaggedAddr(pub SocketAddr);

impl Serialize for TaggedAddr {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        Required::<String, 260>(self.0.to_string()).serialize(s)
    }
}

impl<'de> Deserialize<'de> for TaggedAddr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let wrapped: Required<String, 260> = Required::deserialize(d)?;
        wrapped
            .0
            .parse::<SocketAddr>()
            .map(TaggedAddr)
            .map_err(serde::de::Error::custom)
    }
}

impl From<SocketAddr> for TaggedAddr {
    fn from(addr: SocketAddr) -> Self {
        TaggedAddr(addr)
    }
}

impl From<TaggedAddr> for SocketAddr {
    fn from(tagged: TaggedAddr) -> Self {
        tagged.0
    }
}

impl std::fmt::Display for TaggedAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaggedUuid(pub Uuid);

impl Serialize for TaggedUuid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        Required::<String, 37>(self.0.to_string()).serialize(s)
    }
}

impl<'de> Deserialize<'de> for TaggedUuid {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let wrapped: Required<String, 37> = Required::deserialize(d)?;
        Uuid::parse_str(&wrapped.0)
            .map(TaggedUuid)
            .map_err(serde::de::Error::custom)
    }
}

impl From<Uuid> for TaggedUuid {
    fn from(uuid: Uuid) -> Self {
        TaggedUuid(uuid)
    }
}

impl From<TaggedUuid> for Uuid {
    fn from(tagged: TaggedUuid) -> Self {
        tagged.0
    }
}

impl std::fmt::Display for TaggedUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
