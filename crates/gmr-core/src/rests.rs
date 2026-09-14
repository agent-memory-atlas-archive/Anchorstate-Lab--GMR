use serde::{Deserialize, Serialize};

use crate::addr::ContentHash;
use crate::anchor::{AnchorKey, StatePath};
use crate::probe::FactAddress;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Footprint {
    pub path: StatePath,
    pub hash: ContentHash,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rests {
    pub anchor: AnchorKey,
    pub address: FactAddress,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<Footprint>,
}

impl Rests {
    pub fn on(anchor: AnchorKey, address: FactAddress) -> Self {
        Self {
            anchor,
            address,
            paths: Vec::new(),
        }
    }

    pub fn at(mut self, path: StatePath, hash: ContentHash) -> Self {
        self.paths.push(path_hash(path, hash));
        self
    }

    pub fn whole(&self) -> bool {
        self.paths.is_empty()
    }
}

fn path_hash(path: StatePath, hash: ContentHash) -> Footprint {
    Footprint { path, hash }
}
