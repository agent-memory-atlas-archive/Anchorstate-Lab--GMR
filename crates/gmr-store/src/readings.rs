use async_trait::async_trait;
use gmr_core::{FactAddress, Reading};

use crate::error::StoreError;

#[async_trait]
pub trait Readings: Send + Sync {
    async fn reading(&self, address: &FactAddress) -> Result<Option<Reading>, StoreError>;
}
