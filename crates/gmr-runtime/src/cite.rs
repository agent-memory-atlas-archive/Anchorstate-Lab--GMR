use chrono::{DateTime, Utc};
use gmr_core::{
    AnchorKey, ContentHash, FactAddress, Facts, ProbeVersion, Provenance, Reading, StatePath,
};
use serde::{Deserialize, Serialize};

use crate::assembly::Runtime;
use crate::error::RuntimeError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Look {
    pub anchor: AnchorKey,
    pub taken_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Cited {
    pub address: FactAddress,
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facts: Option<Facts>,
    pub instrument: ProbeVersion,
    pub looks: Vec<Look>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rests {
    pub anchor: AnchorKey,
    pub address: FactAddress,
    #[serde(default)]
    pub paths: Vec<Footprint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Footprint {
    pub path: StatePath,
    pub hash: ContentHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "why", rename_all = "snake_case")]
pub enum Why {
    Value,
    Instrument,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Drifted {
    pub path: StatePath,
    #[serde(flatten)]
    pub why: Why,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "stands", rename_all = "snake_case")]
pub enum Stands {
    Holds,
    Moved {
        drifted: Vec<Drifted>,
    },
    Superseded {
        #[serde(skip_serializing_if = "Option::is_none")]
        now: Option<FactAddress>,
    },
    Unopened,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Upheld {
    pub anchor: AnchorKey,
    pub address: FactAddress,
    #[serde(flatten)]
    pub stands: Stands,
}

impl Runtime {
    pub async fn reading(&self, address: &FactAddress) -> Result<Cited, RuntimeError> {
        let Some(reading) = self.readings.reading(address).await? else {
            return Err(RuntimeError::NoSuchReading {
                address: address.clone(),
            });
        };
        let looks = self
            .scheduler
            .looks_at(address)
            .await?
            .into_iter()
            .map(|seen| Look {
                anchor: seen.anchor,
                taken_at: seen.taken_at,
                provenance: seen.provenance,
            })
            .collect();
        Ok(cited(reading, looks))
    }

    pub async fn stands(&self, asked: &[Rests]) -> Result<Vec<Upheld>, RuntimeError> {
        let mut out = Vec::with_capacity(asked.len());
        for one in asked {
            out.push(Upheld {
                anchor: one.anchor.clone(),
                address: one.address.clone(),
                stands: self.standing_of(one).await?,
            });
        }
        Ok(out)
    }

    async fn standing_of(&self, asked: &Rests) -> Result<Stands, RuntimeError> {
        let Some(now) = self.log.state(&asked.anchor).await? else {
            return Ok(Stands::Unopened);
        };
        let showing = now.latest.as_ref().map(|o| o.fact_address.clone());
        if asked.paths.is_empty() {
            return Ok(match showing.as_ref() == Some(&asked.address) {
                true => Stands::Holds,
                false => Stands::Superseded { now: showing },
            });
        }

        let swapped = match self.readings.reading(&asked.address).await? {
            Some(bound) => now.latest.as_ref().is_some_and(|o| {
                o.versions.derivation.version != bound.versions.derivation.version
            }),
            None => false,
        };

        let drifted: Vec<Drifted> = asked
            .paths
            .iter()
            .filter_map(|footprint| {
                let why = match now.state.hash_at(&footprint.path) {
                    None => Why::Absent,
                    Some(here) if here == footprint.hash => return None,
                    Some(_) if swapped => Why::Instrument,
                    Some(_) => Why::Value,
                };
                Some(Drifted {
                    path: footprint.path.clone(),
                    why,
                })
            })
            .collect();

        Ok(match drifted.is_empty() {
            true => Stands::Holds,
            false => Stands::Moved { drifted },
        })
    }
}

fn cited(reading: Reading, looks: Vec<Look>) -> Cited {
    Cited {
        address: reading.address.clone(),
        found: reading.facts().is_some(),
        facts: reading.facts().cloned(),
        instrument: reading.versions.derivation.version.clone(),
        looks,
    }
}
