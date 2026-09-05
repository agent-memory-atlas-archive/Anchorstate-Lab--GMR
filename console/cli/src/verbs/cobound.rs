use gmr::{Claim, Ref, Runtime};

use crate::error::CliError;

pub async fn run(
    rt: &Runtime,
    names: &crate::memories::Names,
    reference: Ref,
    json: bool,
) -> Result<i32, CliError> {
    let path = names.of(&reference);
    let claim: Claim = reference.clone().into();
    let anchors = rt.memory().binding_of(&claim).await?.anchors().to_vec();
    let others: Vec<Claim> = rt.cobound(&claim).await?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "path": path,
                "anchors": anchors,
                "cobound": others.iter().map(Claim::to_string).collect::<Vec<_>>(),
            })
        );
        return Ok(0);
    }

    match anchors.is_empty() {
        true => println!("{path} is bound to no anchor"),
        false => println!(
            "{path} is bound to {}",
            anchors
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
    if others.is_empty() {
        println!("{path} shares no anchor with any other bound claim");
    } else {
        println!("{path} is co-bound with:");
        for other in &others {
            match other {
                Claim::Stored(reference) => println!("  {}", names.of(reference)),
                said => println!("  {said}"),
            }
        }
    }
    Ok(0)
}
