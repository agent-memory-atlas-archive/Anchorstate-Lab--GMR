use gmr::{FactAddress, Footprint, Rests, Runtime, StatePath};

use crate::error::CliError;

pub async fn reading(rt: &Runtime, address: String, json: bool) -> Result<i32, CliError> {
    let cited = rt.reading(&addressed(&address)?).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&cited)?);
        return Ok(0);
    }
    println!(
        "{}  {}",
        &cited.address.as_str()[..12],
        cited.instrument.short()
    );
    match &cited.facts {
        Some(facts) => println!("{}", serde_json::to_string_pretty(facts.as_value())?),
        None => println!("not found when it was read"),
    }
    for look in &cited.looks {
        match &look.provenance {
            Some(who) => println!("  {} {} {who}", look.taken_at, look.anchor),
            None => println!("  {} {}", look.taken_at, look.anchor),
        }
    }
    Ok(0)
}

pub async fn stands(rt: &Runtime, cited: Vec<String>, json: bool) -> Result<i32, CliError> {
    let mut asked = Vec::with_capacity(cited.len());
    for one in &cited {
        asked.push(rests(rt, one).await?);
    }
    let held = rt.stands(&asked).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&held)?);
        return Ok(0);
    }
    for one in &held {
        println!(
            "{}  {}  {}",
            &one.address.as_str()[..12],
            one.anchor,
            serde_json::to_string(&one.stands)?
        );
    }
    Ok(held.iter().any(|o| o.stands != gmr::Stands::Holds) as i32)
}

async fn rests(rt: &Runtime, spelled: &str) -> Result<Rests, CliError> {
    let (anchor, rest) = spelled.split_once('@').ok_or_else(|| {
        CliError(format!(
            "`{spelled}` does not name a citation. Spell it `<anchor>@<address>` for the \
             whole reading, or `<anchor>@<address>#<path>` for one path of the state"
        ))
    })?;
    let (address, path) = match rest.split_once('#') {
        Some((address, path)) => (address, Some(path)),
        None => (rest, None),
    };
    let anchor = gmr::AnchorKey::new(anchor);
    let address = addressed(address)?;
    let Some(path) = path else {
        return Ok(Rests {
            anchor,
            address,
            paths: Vec::new(),
        });
    };
    let path = StatePath::try_new(path).map_err(|e| CliError(format!("`{path}`: {e}")))?;
    let hash = rt
        .read(&anchor)
        .await?
        .state
        .hash_at(&path)
        .ok_or_else(|| {
            CliError(format!(
                "`{anchor}` carries nothing at `{path}` right now, so there is no hash to \
                 stand a citation on. An assertion already resting on it would answer \
                 `absent`; ask `gmr stands` through the assertion, not by hand"
            ))
        })?;
    Ok(Rests {
        anchor,
        address,
        paths: vec![Footprint { path, hash }],
    })
}

fn addressed(address: &str) -> Result<FactAddress, CliError> {
    FactAddress::try_new(address.to_owned()).map_err(|e| {
        CliError(format!(
            "`{address}` is not a fact address ({e}). An address is the sha256 a read entry \
             issued for a reading: 64 hex characters"
        ))
    })
}
