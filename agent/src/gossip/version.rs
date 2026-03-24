use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct Version {
    pub(super) counter: u64,
    pub(super) generation: u64,
}
