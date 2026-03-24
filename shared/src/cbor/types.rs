use std::net::SocketAddr;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
