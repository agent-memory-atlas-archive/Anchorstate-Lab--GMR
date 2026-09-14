use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gmr_core::{AnchorKey, FactAddress, Sighting};

use crate::error::StoreError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Seen {
    pub sightings: u64,
    pub last_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait Sightings: Send + Sync {
    async fn sighted(&self, sighting: &Sighting) -> Result<(), StoreError>;

    async fn seen(&self, anchor: &AnchorKey) -> Result<Seen, StoreError>;

    async fn all_seen(&self) -> Result<BTreeMap<AnchorKey, Seen>, StoreError>;

    async fn of(&self, address: &FactAddress) -> Result<Vec<Sighting>, StoreError>;
}
