use gmr::{Instructions, Runtime};

use crate::{error::CliError, render};

pub async fn run(
    rt: &Runtime,
    names: &crate::memories::Names,
    key: Option<String>,
    fresher_than_secs: Option<u64>,
    lean: bool,
    carry: bool,
    json: bool,
) -> Result<i32, CliError> {
    let how = Instructions {
        max_staleness: fresher_than_secs.map(std::time::Duration::from_secs),
        lean,
        carry,
        ..Instructions::default()
    };
    let views = match key {
        Some(k) => {
            let mut out = Vec::new();
            for key in super::resolve(rt, &k).await? {
                out.push(rt.grounded_within(&key, &how).await?);
            }
            out
        }
        None => rt.grounded_all_within(&how).await?,
    };

    if json {
        let mut out = serde_json::to_value(&views)?;
        if let Some(items) = out.as_array_mut() {
            for item in items {
                render::rules_as_hash(item)?;
            }
        }
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else if views.is_empty() {
        println!("no anchors.");
    } else {
        for v in &views {
            print!("{}", render::anchor(v, names));
        }
    }
    Ok(0)
}
