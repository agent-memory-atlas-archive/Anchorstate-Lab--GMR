use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::addr::{ContentHash, content_hash_of};
use crate::probe::ProbeRef;
use crate::string_newtype;

pub const POSITION: &str = "position";

pub const STATUS: &str = "status";

fn check_key(s: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err("must not be empty".to_owned());
    }
    if s.len() > 128 {
        return Err("must be at most 128 chars".to_owned());
    }
    Ok(())
}

fn check_status(s: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err("must not be empty".to_owned());
    }
    if s.len() > 64 {
        return Err("must be at most 64 chars".to_owned());
    }
    Ok(())
}

string_newtype! {
    admitted AnchorKey, check_key
}

string_newtype! {
    admitted StatusId, check_status
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expr {
    pub source: String,
    pub hash: ContentHash,
}

impl Expr {
    pub fn text(source: impl Into<String>) -> Self {
        let source = source.into();
        let hash = content_hash_of(&Value::String(source.clone()))
            .expect("Value::String never exceeds canonicalization limits");
        Self { source, hash }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub when: Expr,
    pub to: Expr,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Transitions(pub Vec<Rule>);

impl Transitions {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct State(pub Value);

impl State {
    pub fn new(v: Value) -> Self {
        Self(v)
    }

    pub fn position(&self) -> &Value {
        self.0.get(POSITION).unwrap_or(&Value::Null)
    }

    pub fn status(&self) -> Option<StatusId> {
        self.0
            .get(STATUS)
            .and_then(Value::as_str)
            .map(StatusId::new)
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }

    pub fn at(&self, path: &StatePath) -> Option<&Value> {
        let mut here = &self.0;
        for step in path.steps() {
            here = match here {
                Value::Object(fields) => fields.get(step)?,
                Value::Array(items) => items.get(step.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        Some(here)
    }

    pub fn hash_at(&self, path: &StatePath) -> Option<ContentHash> {
        self.at(path).and_then(|v| content_hash_of(v).ok())
    }
}

fn check_path(s: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err("must not be empty".to_owned());
    }
    if s.len() > 256 {
        return Err("must be at most 256 chars".to_owned());
    }
    if s.split('.').any(str::is_empty) {
        return Err(format!(
            "`{s}` has an empty step; steps are separated by `.`"
        ));
    }
    Ok(())
}

string_newtype! {
    admitted StatePath, check_path
}

impl StatePath {
    pub fn steps(&self) -> impl Iterator<Item = &str> {
        self.as_str().split('.')
    }
}

impl Default for State {
    fn default() -> Self {
        Self(Value::Object(serde_json::Map::new()))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Retain {
    #[default]
    Tick,
    Full,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Recorded {
    #[default]
    Plain,
    Digests,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSettings {
    #[serde(default)]
    pub retain: Retain,
    #[serde(default)]
    pub facts: Recorded,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cadence_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_ms: Option<u64>,
}

impl RunSettings {
    pub fn retains_full(&self) -> bool {
        matches!(self.retain, Retain::Full)
    }

    pub fn records_digests_only(&self) -> bool {
        matches!(self.facts, Recorded::Digests)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Superseded {
    pub key: AnchorKey,
    pub rationale: ContentHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    pub key: AnchorKey,
    pub probe: ProbeRef,
    pub transitions: Transitions,
    #[serde(default)]
    pub terminal: BTreeSet<StatusId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<Superseded>,
}

impl Anchor {
    pub fn is_terminal(&self, state: &State) -> bool {
        state.status().is_some_and(|s| self.terminal.contains(&s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn anchor(terminal: &[&str]) -> Anchor {
        Anchor {
            key: AnchorKey::new("a"),
            probe: crate::probe::ProbeRef::new(
                crate::probe::Kind::new("shell"),
                crate::probe::ProbeName::new("p"),
                json!({}),
            ),
            transitions: Transitions::default(),
            terminal: terminal.iter().map(|s| StatusId::new(*s)).collect(),
            supersedes: None,
        }
    }

    fn path(s: &str) -> StatePath {
        StatePath::try_new(s).expect("the fixture spells a path")
    }

    #[test]
    fn a_path_walks_objects_and_arrays_and_stops_where_the_shape_ends() {
        let s = State::new(json!({ "now": { "sig": "fn a()", "members": ["x", "y"] } }));
        assert_eq!(s.at(&path("now.sig")), Some(&json!("fn a()")));
        assert_eq!(s.at(&path("now.members.1")), Some(&json!("y")));
        assert_eq!(s.at(&path("now.members.9")), None);
        assert_eq!(s.at(&path("now.sig.deeper")), None);
        assert_eq!(s.at(&path("absent")), None);
    }

    #[test]
    fn one_path_moving_leaves_every_other_path_hash_alone() {
        let before = State::new(json!({ "now": { "sig": "fn a()", "place": 10 } }));
        let after = State::new(json!({ "now": { "sig": "fn a()", "place": 44 } }));
        assert_eq!(
            before.hash_at(&path("now.sig")),
            after.hash_at(&path("now.sig")),
            "an edit elsewhere in the file must not stale a memory about the signature"
        );
        assert_ne!(
            before.hash_at(&path("now.place")),
            after.hash_at(&path("now.place"))
        );
    }

    #[test]
    fn a_path_that_is_gone_hashes_to_nothing_rather_than_to_null() {
        let s = State::new(json!({ "now": { "sig": serde_json::Value::Null } }));
        assert!(
            s.hash_at(&path("now.sig")).is_some(),
            "a null value is a value"
        );
        assert!(s.hash_at(&path("now.gone")).is_none());
    }

    #[test]
    fn a_path_with_an_empty_step_is_refused_because_it_would_silently_match_the_root() {
        assert!(StatePath::try_new("now..sig").is_err());
        assert!(StatePath::try_new("").is_err());
        assert!(StatePath::try_new(".now").is_err());
        assert!(StatePath::try_new("now.sig").is_ok());
    }

    #[test]
    fn expression_identity_is_its_hash() {
        assert_eq!(Expr::text("obs.a").hash, Expr::text("obs.a").hash);
        assert_ne!(Expr::text("obs.a").hash, Expr::text("obs.b").hash);
    }

    #[test]
    fn position_is_a_slot_the_substrate_carries_but_never_reads_into() {
        let s = State::new(json!({ POSITION: { "file": "a.rs", "symbol": "assess" } }));
        assert_eq!(s.position(), &json!({ "file": "a.rs", "symbol": "assess" }));
    }

    #[test]
    fn a_state_without_a_position_is_legal() {
        assert_eq!(State::default().position(), &Value::Null);
    }

    #[test]
    fn terminal_is_decided_by_the_status_slot_alone() {
        let a = anchor(&["settled", "expired"]);
        assert!(a.is_terminal(&State::new(json!({ STATUS: "settled" }))));
        assert!(!a.is_terminal(&State::new(json!({ STATUS: "drifted" }))));
    }

    #[test]
    fn a_state_with_no_status_is_never_terminal() {
        let a = anchor(&["settled"]);
        assert!(!a.is_terminal(&State::default()));
        assert!(!a.is_terminal(&State::new(json!({ POSITION: "somewhere" }))));
    }

    #[test]
    fn the_substrate_does_not_read_into_the_status() {
        let a = anchor(&["расчёт"]);
        assert!(a.is_terminal(&State::new(json!({ STATUS: "расчёт" }))));
    }

    #[test]
    fn states_compare_by_content() {
        let a = State::new(json!({ "status": "ok", "count": 2 }));
        let b = State::new(json!({ "count": 2, "status": "ok" }));
        assert_eq!(a, b);
    }

    #[test]
    fn anchor_roundtrips_the_wire() {
        let a = anchor(&["settled"]);
        let s = serde_json::to_string(&a).unwrap();
        assert_eq!(serde_json::from_str::<Anchor>(&s).unwrap(), a);
    }
}
