use async_trait::async_trait;
use gmr_core::{Entry, FactAddress, Reading};
use sqlx::Row;

use super::journal::SqliteJournal;
use super::{db_err, decode_err};
use crate::{Readings, StoreError};

#[async_trait]
impl Readings for SqliteJournal {
    async fn reading(&self, address: &FactAddress) -> Result<Option<Reading>, StoreError> {
        let row = sqlx::query(
            "SELECT body FROM journal
             WHERE json_extract(body, '$.observation.fact_address') = ?1
             ORDER BY seq LIMIT 1",
        )
        .bind(address.as_str())
        .fetch_optional(self.pool())
        .await
        .map_err(db_err)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let body: String = row.get("body");
        let entry: Entry = serde_json::from_str(&body).map_err(decode_err)?;
        Ok(match entry {
            Entry::Open { observation, .. } | Entry::Transition { observation, .. } => {
                Some(Reading::of(&observation))
            }
            _ => None,
        })
    }
}
