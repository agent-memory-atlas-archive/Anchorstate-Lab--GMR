use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gmr_core::{AnchorKey, FactAddress, Provenance, Sighting};
use sqlx::Row;

use super::db_err;
use super::queue::SqliteQueue;
use crate::{Seen, Sightings, StoreError};

pub(crate) fn moment(text: Option<String>) -> Result<Option<DateTime<Utc>>, StoreError> {
    text.map(|t| {
        DateTime::parse_from_rfc3339(&t)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| {
                StoreError::corrupt(format!(
                    "sighting.taken_at holds {t:?}, which is not a time: {e}"
                ))
            })
    })
    .transpose()
}

fn taken(text: String) -> Result<DateTime<Utc>, StoreError> {
    moment(Some(text))?.ok_or_else(|| StoreError::corrupt("a sighting with no instant".to_owned()))
}

#[async_trait]
impl Sightings for SqliteQueue {
    async fn sighted(&self, sighting: &Sighting) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO sighting (anchor, address, taken_at, provenance)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(sighting.anchor.as_str())
        .bind(sighting.address.as_str())
        .bind(sighting.taken_at.to_rfc3339())
        .bind(sighting.provenance.as_ref().map(Provenance::as_str))
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    async fn seen(&self, anchor: &AnchorKey) -> Result<Seen, StoreError> {
        let row = sqlx::query(
            "SELECT count(*) AS looks, max(taken_at) AS last_at
             FROM sighting WHERE anchor = ?1",
        )
        .bind(anchor.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;

        Ok(Seen {
            sightings: row.get::<i64, _>("looks") as u64,
            last_at: moment(row.get::<Option<String>, _>("last_at"))?,
        })
    }

    async fn all_seen(&self) -> Result<BTreeMap<AnchorKey, Seen>, StoreError> {
        let rows = sqlx::query(
            "SELECT anchor, count(*) AS looks, max(taken_at) AS last_at
             FROM sighting GROUP BY anchor",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;

        rows.into_iter()
            .map(|r| {
                Ok((
                    AnchorKey::new(r.get::<String, _>("anchor")),
                    Seen {
                        sightings: r.get::<i64, _>("looks") as u64,
                        last_at: moment(r.get::<Option<String>, _>("last_at"))?,
                    },
                ))
            })
            .collect()
    }

    async fn of(&self, address: &FactAddress) -> Result<Vec<Sighting>, StoreError> {
        let rows = sqlx::query(
            "SELECT anchor, address, taken_at, provenance
             FROM sighting WHERE address = ?1 ORDER BY seq",
        )
        .bind(address.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;

        rows.into_iter()
            .map(|r| {
                let seen = Sighting::new(
                    AnchorKey::new(r.get::<String, _>("anchor")),
                    FactAddress::try_new(r.get::<String, _>("address"))
                        .map_err(|e| StoreError::corrupt(e.to_string()))?,
                    taken(r.get::<String, _>("taken_at"))?,
                );
                Ok(match r.get::<Option<String>, _>("provenance") {
                    Some(who) => seen.by(Provenance::new(who)),
                    None => seen,
                })
            })
            .collect()
    }
}
