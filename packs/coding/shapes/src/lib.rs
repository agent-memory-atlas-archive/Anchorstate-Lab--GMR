use gmr_core::{Expr, Rule, Transitions};

#[derive(Debug)]
pub struct Shape {
    pub name: &'static str,
    dims: &'static [Dim],
    watch: &'static [&'static str],
    dwell: bool,
}

impl Shape {
    pub fn dwells(&self) -> bool {
        self.dwell
    }

    pub fn dims(&self) -> &'static [Dim] {
        self.dims
    }
}

#[derive(Debug)]
pub struct Dim {
    pub name: &'static str,
    pub status: &'static str,
    reads: Reads,
}

#[derive(Debug)]
enum Reads {
    Now {
        guard: &'static str,
    },
    Since {
        field: &'static str,
        obs: &'static str,
        op: Cmp,
    },
}

#[derive(Debug, Clone, Copy)]
enum Cmp {
    Ne,
    Gt,
    Lt,
}

impl Cmp {
    fn as_str(self) -> &'static str {
        match self {
            Cmp::Ne => "!=",
            Cmp::Gt => ">",
            Cmp::Lt => "<",
        }
    }
}

const fn now(name: &'static str, status: &'static str, guard: &'static str) -> Dim {
    Dim {
        name,
        status,
        reads: Reads::Now { guard },
    }
}

const fn since(
    name: &'static str,
    status: &'static str,
    field: &'static str,
    obs: &'static str,
) -> Dim {
    Dim {
        name,
        status,
        reads: Reads::Since {
            field,
            obs,
            op: Cmp::Ne,
        },
    }
}

const fn compare(
    name: &'static str,
    status: &'static str,
    field: &'static str,
    obs: &'static str,
    op: Cmp,
) -> Dim {
    Dim {
        name,
        status,
        reads: Reads::Since { field, obs, op },
    }
}

const GONE: Dim = now(MISSING, "missing", "obs.found == false");

const CONTRACT_DIMS: &[Dim] = &[
    GONE,
    since("name", "renamed", "name", "obs.at.name"),
    since("file", "relocated", "file", "obs.at.file"),
    since("kind", "kind-changed", "form", "obs.at.form"),
    since("sig", "signature-changed", "sig", "obs.at.shape"),
    since("surface", "surface-changed", "surface", "obs.at.surface"),
    since("logic", "logic-changed", "body", "obs.facts.body"),
    since("place", "moved", "after", "obs.at.after"),
];

const CONTRACT_WATCH: &[&str] = &[
    "missing", "name", "file", "kind", "sig", "surface", "logic", "place",
];

const CONTRACT: Shape = Shape {
    name: "contract",
    dims: CONTRACT_DIMS,
    watch: CONTRACT_WATCH,
    dwell: false,
};

const CONTRACT_DWELL: Shape = Shape {
    name: "contract-dwell",
    dims: CONTRACT_DIMS,
    watch: CONTRACT_WATCH,
    dwell: true,
};

const ROSTER_DIMS: &[Dim] = &[
    GONE,
    compare("grew", "grew", "count", "obs.candidates", Cmp::Gt),
    compare("shrank", "shrank", "count", "obs.candidates", Cmp::Lt),
    since("roll", "swapped", "roll", "obs.roll"),
];

const ROSTER_WATCH: &[&str] = &["missing", "grew", "shrank", "roll"];

const ROSTER: Shape = Shape {
    name: "roster",
    dims: ROSTER_DIMS,
    watch: ROSTER_WATCH,
    dwell: false,
};

const ROSTER_DWELL: Shape = Shape {
    name: "roster-dwell",
    dims: ROSTER_DIMS,
    watch: ROSTER_WATCH,
    dwell: true,
};

const FINGERPRINT_DIMS: &[Dim] = &[
    GONE,
    since("drift", "drifted", "fingerprint", "obs.at.fingerprint"),
];

const FINGERPRINT_WATCH: &[&str] = &["missing", "drift"];

const FINGERPRINT: Shape = Shape {
    name: "fingerprint",
    dims: FINGERPRINT_DIMS,
    watch: FINGERPRINT_WATCH,
    dwell: false,
};

const FINGERPRINT_DWELL: Shape = Shape {
    name: "fingerprint-dwell",
    dims: FINGERPRINT_DIMS,
    watch: FINGERPRINT_WATCH,
    dwell: true,
};

const VALUE_DIMS: &[Dim] = &[since("value", "changed", "value", "obs.value")];

const VALUE_WATCH: &[&str] = &["value"];

const VALUE: Shape = Shape {
    name: "value",
    dims: VALUE_DIMS,
    watch: VALUE_WATCH,
    dwell: false,
};

const VALUE_DWELL: Shape = Shape {
    name: "value-dwell",
    dims: VALUE_DIMS,
    watch: VALUE_WATCH,
    dwell: true,
};

pub const ALL: &[&Shape] = &[
    &CONTRACT,
    &ROSTER,
    &FINGERPRINT,
    &VALUE,
    &CONTRACT_DWELL,
    &ROSTER_DWELL,
    &FINGERPRINT_DWELL,
    &VALUE_DWELL,
];

pub const DWELL: &str = "15m";

pub const MISSING: &str = "missing";

pub const SETTLED: &str = "settled";

pub const ABSENT: &str = "absent";

pub const FLUX: &str = "flux";

pub const RETURNED: &str = "returned";

pub const REPLACED: &str = "replaced";

pub const RETIRED: &[&str] = &[
    "added",
    "captured",
    "count-moved",
    "matches",
    "moved-file",
    "occurrence",
    "removed",
    "section-gone",
    "symbol",
];

pub fn get(name: &str) -> Result<&'static Shape, String> {
    ALL.iter().copied().find(|s| s.name == name).ok_or_else(|| {
        format!(
            "unknown shape `{name}`; this build ships {}",
            ALL.iter().map(|s| s.name).collect::<Vec<_>>().join(" · ")
        )
    })
}

pub fn rules(shape: &Shape) -> Vec<(String, String)> {
    match shape.dwell {
        true => expand_dwell(shape.dims),
        false => expand(shape.dims),
    }
}

pub fn rules_of(shape: &Shape) -> Vec<String> {
    rules(shape)
        .into_iter()
        .map(|(when, to)| format!("{when} => {to}"))
        .collect()
}

pub fn transitions_of(shape: &Shape) -> Transitions {
    Transitions(
        rules(shape)
            .into_iter()
            .map(|(when, to)| Rule {
                when: Expr::text(when),
                to: Expr::text(to),
            })
            .collect(),
    )
}

pub fn of(transitions: &Transitions) -> Option<&'static Shape> {
    ALL.iter()
        .copied()
        .find(|s| &transitions_of(s) == transitions)
}

pub fn name_of(transitions: &Transitions) -> Option<&'static str> {
    of(transitions).map(|s| s.name)
}

pub fn watch_of(shape: &Shape) -> &'static [&'static str] {
    shape.watch
}

pub fn axes_of(shape: &Shape) -> Vec<&'static str> {
    shape.dims.iter().map(|d| d.name).collect()
}

fn quoted(s: &str) -> String {
    format!("\"{s}\"")
}

fn object(fields: &[(&str, &str)]) -> String {
    let body: Vec<String> = fields.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    format!("{{ {} }}", body.join(", "))
}

fn reading(dims: &[Dim]) -> String {
    let mut fields: Vec<(&str, &str)> = Vec::new();
    for d in dims {
        let Reads::Since { field, obs, .. } = d.reads else {
            continue;
        };
        if !fields.iter().any(|(k, _)| *k == field) {
            fields.push((field, obs));
        }
    }
    object(&fields)
}

fn vector(dims: &[Dim], each: impl Fn(&Dim) -> String) -> String {
    let bits: Vec<(&str, String)> = dims.iter().map(|d| (d.name, each(d))).collect();
    object(
        &bits
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect::<Vec<_>>(),
    )
}

fn bit(d: &Dim) -> String {
    match d.reads {
        Reads::Now { guard } => guard.to_owned(),
        Reads::Since { field, obs, op } => format!(
            "state.v.{} or ({obs} {} state.baseline.{field})",
            d.name,
            op.as_str()
        ),
    }
}

fn opening(d: &Dim) -> String {
    match d.reads {
        Reads::Now { guard } => guard.to_owned(),
        Reads::Since { .. } => "false".into(),
    }
}

fn carried(d: &Dim) -> String {
    match d.reads {
        Reads::Now { guard } => guard.to_owned(),
        Reads::Since { .. } => format!("state.v.{}", d.name),
    }
}

fn seen(dims: &[Dim], status: &str) -> String {
    object(&[
        ("position", "state.position"),
        ("baseline", "state.baseline"),
        ("now", &reading(dims)),
        ("v", &vector(dims, bit)),
        ("status", &quoted(status)),
    ])
}

fn reported(dims: &[Dim]) -> String {
    if dims.iter().any(|d| d.name == MISSING) {
        return "obs.found".to_owned();
    }
    let mut guards: Vec<String> = Vec::new();
    for d in dims {
        let Reads::Since { obs, .. } = d.reads else {
            continue;
        };
        let guard = format!("exists({obs})");
        if !guards.contains(&guard) {
            guards.push(guard);
        }
    }
    match guards.is_empty() {
        true => "true".to_owned(),
        false => guards.join(" and "),
    }
}

fn expand(dims: &[Dim]) -> Vec<(String, String)> {
    let standing = || dims.iter().filter(|d| matches!(d.reads, Reads::Now { .. }));
    let mut out = Vec::with_capacity(dims.len() + 3);

    out.push((
        format!("not exists(state.baseline) and {}", reported(dims)),
        object(&[
            ("position", "state.position"),
            ("baseline", &reading(dims)),
            ("now", &reading(dims)),
            ("v", &vector(dims, opening)),
            ("status", &quoted(SETTLED)),
        ]),
    ));

    out.push((
        "not exists(state.baseline)".to_owned(),
        object(&[
            ("position", "state.position"),
            ("v", &vector(dims, opening)),
            ("status", &quoted(ABSENT)),
        ]),
    ));

    out.extend(standing().map(|d| {
        (
            bit(d),
            object(&[
                ("position", "state.position"),
                ("baseline", "state.baseline"),
                ("now", "state.now"),
                ("v", &vector(dims, carried)),
                ("status", &quoted(d.status)),
            ]),
        )
    }));

    out.extend(
        dims.iter()
            .filter(|d| matches!(d.reads, Reads::Since { .. }))
            .map(|d| (bit(d), seen(dims, d.status))),
    );
    out.push(("true".to_owned(), seen(dims, SETTLED)));
    out
}

#[derive(Clone, Copy)]
enum Counters {
    Bump,
    Carry,
}

fn counted(counters: Counters) -> (&'static str, &'static str) {
    match counters {
        Counters::Bump => ("state.moved_count + 1", "taken_at"),
        Counters::Carry => ("state.moved_count", "state.last_moved_at"),
    }
}

fn seen_dwell(dims: &[Dim], status: &str, phase: &str, counters: Counters) -> String {
    let (count, at) = counted(counters);
    object(&[
        ("position", "state.position"),
        ("baseline", "state.baseline"),
        ("now", &reading(dims)),
        ("v", &vector(dims, bit)),
        ("status", &quoted(status)),
        ("phase", &quoted(phase)),
        ("moved_count", count),
        ("last_moved_at", at),
        ("last_present", &reading(dims)),
    ])
}

fn changed_now(dims: &[Dim]) -> String {
    let mut terms: Vec<String> = Vec::new();
    for d in dims {
        let Reads::Since { field, obs, .. } = d.reads else {
            continue;
        };
        let term = format!("{obs} != state.now.{field}");
        if !terms.contains(&term) {
            terms.push(term);
        }
    }
    match terms.is_empty() {
        true => "false".to_owned(),
        false => terms.join(" or "),
    }
}

fn identity(dims: &[Dim]) -> String {
    let since: Vec<&Dim> = dims
        .iter()
        .filter(|d| matches!(d.reads, Reads::Since { .. }))
        .collect();
    let named = since
        .iter()
        .find(|d| matches!(d.reads, Reads::Since { field: "sig", .. }))
        .or(since.last())
        .expect("a shape compares at least one field");
    let Reads::Since { field, obs, .. } = named.reads else {
        unreachable!()
    };
    format!("{obs} == state.last_present.{field}")
}

fn expand_dwell(dims: &[Dim]) -> Vec<(String, String)> {
    let standing = || dims.iter().filter(|d| matches!(d.reads, Reads::Now { .. }));
    let mut out = Vec::with_capacity(dims.len() * 3 + 8);

    out.push((
        "exists(state.baseline) and not exists(state.moved_count)".to_owned(),
        object(&[
            ("position", "state.position"),
            ("baseline", "state.baseline"),
            ("now", "state.now"),
            ("v", "state.v"),
            ("status", "state.status"),
            ("phase", &quoted(SETTLED)),
            ("moved_count", "0"),
            ("last_moved_at", "taken_at"),
            ("last_present", "state.now"),
        ]),
    ));

    out.push((
        format!("not exists(state.baseline) and {}", reported(dims)),
        object(&[
            ("position", "state.position"),
            ("baseline", &reading(dims)),
            ("now", &reading(dims)),
            ("v", &vector(dims, opening)),
            ("status", &quoted(SETTLED)),
            ("phase", &quoted(SETTLED)),
            ("moved_count", "0"),
            ("last_moved_at", "taken_at"),
            ("last_present", &reading(dims)),
        ]),
    ));

    out.push((
        "not exists(state.baseline) and exists(state.moved_count)".to_owned(),
        object(&[
            ("position", "state.position"),
            ("v", &vector(dims, opening)),
            ("status", &quoted(ABSENT)),
            ("phase", &quoted(ABSENT)),
            ("moved_count", "state.moved_count"),
            ("last_moved_at", "state.last_moved_at"),
        ]),
    ));

    out.push((
        "not exists(state.baseline)".to_owned(),
        object(&[
            ("position", "state.position"),
            ("v", &vector(dims, opening)),
            ("status", &quoted(ABSENT)),
            ("phase", &quoted(ABSENT)),
            ("moved_count", "0"),
            ("last_moved_at", "taken_at"),
        ]),
    ));

    for d in standing() {
        out.push((
            format!("({}) and state.phase != {}", bit(d), quoted(ABSENT)),
            object(&[
                ("position", "state.position"),
                ("baseline", "state.baseline"),
                ("now", "state.now"),
                ("v", &vector(dims, carried)),
                ("status", &quoted(d.status)),
                ("phase", &quoted(ABSENT)),
                ("moved_count", "state.moved_count + 1"),
                ("last_moved_at", "taken_at"),
                ("last_present", "state.now"),
            ]),
        ));
        out.push((
            bit(d),
            object(&[
                ("position", "state.position"),
                ("baseline", "state.baseline"),
                ("now", "state.now"),
                ("v", &vector(dims, carried)),
                ("status", &quoted(d.status)),
                ("phase", &quoted(ABSENT)),
                ("moved_count", "state.moved_count"),
                ("last_moved_at", "state.last_moved_at"),
                ("last_present", "state.last_present"),
            ]),
        ));
    }

    if standing().next().is_some() {
        let back = format!(
            "exists(state.last_present) and state.phase == {} and {}",
            quoted(ABSENT),
            reported(dims)
        );
        out.push((
            format!("{back} and {}", identity(dims)),
            seen_dwell(dims, RETURNED, RETURNED, Counters::Bump),
        ));
        out.push((back, seen_dwell(dims, REPLACED, REPLACED, Counters::Bump)));
    }

    let changed = changed_now(dims);
    let within = format!(
        "state.phase == {} and taken_at - state.last_moved_at < {DWELL}",
        quoted(FLUX)
    );
    for d in dims
        .iter()
        .filter(|d| matches!(d.reads, Reads::Since { .. }))
    {
        out.push((
            format!("({}) and ({changed})", bit(d)),
            seen_dwell(dims, d.status, FLUX, Counters::Bump),
        ));
        out.push((
            format!("({}) and {within}", bit(d)),
            seen_dwell(dims, d.status, FLUX, Counters::Carry),
        ));
        out.push((bit(d), seen_dwell(dims, d.status, SETTLED, Counters::Carry)));
    }
    out.push((within, seen_dwell(dims, SETTLED, FLUX, Counters::Carry)));
    out.push((
        "true".to_owned(),
        seen_dwell(dims, SETTLED, SETTLED, Counters::Carry),
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use gmr_expr::{Ctx, Evaluated};
    use gmr_survey::matching::COORD_REPORT_SCHEMA as COORD_SCHEMA;
    use serde_json::Value;
    use std::collections::BTreeSet;

    const SHIPPED_CONTRACT: &[&str] = &[
        "not exists(state.baseline) and obs.found => { position: state.position, baseline: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, now: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, v: { missing: obs.found == false, name: false, file: false, kind: false, sig: false, surface: false, logic: false, place: false }, status: \"settled\" }",
        "not exists(state.baseline) => { position: state.position, v: { missing: obs.found == false, name: false, file: false, kind: false, sig: false, surface: false, logic: false, place: false }, status: \"absent\" }",
        "obs.found == false => { position: state.position, baseline: state.baseline, now: state.now, v: { missing: obs.found == false, name: state.v.name, file: state.v.file, kind: state.v.kind, sig: state.v.sig, surface: state.v.surface, logic: state.v.logic, place: state.v.place }, status: \"missing\" }",
        "state.v.name or (obs.at.name != state.baseline.name) => { position: state.position, baseline: state.baseline, now: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, v: { missing: obs.found == false, name: state.v.name or (obs.at.name != state.baseline.name), file: state.v.file or (obs.at.file != state.baseline.file), kind: state.v.kind or (obs.at.form != state.baseline.form), sig: state.v.sig or (obs.at.shape != state.baseline.sig), surface: state.v.surface or (obs.at.surface != state.baseline.surface), logic: state.v.logic or (obs.facts.body != state.baseline.body), place: state.v.place or (obs.at.after != state.baseline.after) }, status: \"renamed\" }",
        "state.v.file or (obs.at.file != state.baseline.file) => { position: state.position, baseline: state.baseline, now: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, v: { missing: obs.found == false, name: state.v.name or (obs.at.name != state.baseline.name), file: state.v.file or (obs.at.file != state.baseline.file), kind: state.v.kind or (obs.at.form != state.baseline.form), sig: state.v.sig or (obs.at.shape != state.baseline.sig), surface: state.v.surface or (obs.at.surface != state.baseline.surface), logic: state.v.logic or (obs.facts.body != state.baseline.body), place: state.v.place or (obs.at.after != state.baseline.after) }, status: \"relocated\" }",
        "state.v.kind or (obs.at.form != state.baseline.form) => { position: state.position, baseline: state.baseline, now: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, v: { missing: obs.found == false, name: state.v.name or (obs.at.name != state.baseline.name), file: state.v.file or (obs.at.file != state.baseline.file), kind: state.v.kind or (obs.at.form != state.baseline.form), sig: state.v.sig or (obs.at.shape != state.baseline.sig), surface: state.v.surface or (obs.at.surface != state.baseline.surface), logic: state.v.logic or (obs.facts.body != state.baseline.body), place: state.v.place or (obs.at.after != state.baseline.after) }, status: \"kind-changed\" }",
        "state.v.sig or (obs.at.shape != state.baseline.sig) => { position: state.position, baseline: state.baseline, now: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, v: { missing: obs.found == false, name: state.v.name or (obs.at.name != state.baseline.name), file: state.v.file or (obs.at.file != state.baseline.file), kind: state.v.kind or (obs.at.form != state.baseline.form), sig: state.v.sig or (obs.at.shape != state.baseline.sig), surface: state.v.surface or (obs.at.surface != state.baseline.surface), logic: state.v.logic or (obs.facts.body != state.baseline.body), place: state.v.place or (obs.at.after != state.baseline.after) }, status: \"signature-changed\" }",
        "state.v.surface or (obs.at.surface != state.baseline.surface) => { position: state.position, baseline: state.baseline, now: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, v: { missing: obs.found == false, name: state.v.name or (obs.at.name != state.baseline.name), file: state.v.file or (obs.at.file != state.baseline.file), kind: state.v.kind or (obs.at.form != state.baseline.form), sig: state.v.sig or (obs.at.shape != state.baseline.sig), surface: state.v.surface or (obs.at.surface != state.baseline.surface), logic: state.v.logic or (obs.facts.body != state.baseline.body), place: state.v.place or (obs.at.after != state.baseline.after) }, status: \"surface-changed\" }",
        "state.v.logic or (obs.facts.body != state.baseline.body) => { position: state.position, baseline: state.baseline, now: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, v: { missing: obs.found == false, name: state.v.name or (obs.at.name != state.baseline.name), file: state.v.file or (obs.at.file != state.baseline.file), kind: state.v.kind or (obs.at.form != state.baseline.form), sig: state.v.sig or (obs.at.shape != state.baseline.sig), surface: state.v.surface or (obs.at.surface != state.baseline.surface), logic: state.v.logic or (obs.facts.body != state.baseline.body), place: state.v.place or (obs.at.after != state.baseline.after) }, status: \"logic-changed\" }",
        "state.v.place or (obs.at.after != state.baseline.after) => { position: state.position, baseline: state.baseline, now: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, v: { missing: obs.found == false, name: state.v.name or (obs.at.name != state.baseline.name), file: state.v.file or (obs.at.file != state.baseline.file), kind: state.v.kind or (obs.at.form != state.baseline.form), sig: state.v.sig or (obs.at.shape != state.baseline.sig), surface: state.v.surface or (obs.at.surface != state.baseline.surface), logic: state.v.logic or (obs.facts.body != state.baseline.body), place: state.v.place or (obs.at.after != state.baseline.after) }, status: \"moved\" }",
        "true => { position: state.position, baseline: state.baseline, now: { name: obs.at.name, file: obs.at.file, form: obs.at.form, sig: obs.at.shape, surface: obs.at.surface, body: obs.facts.body, after: obs.at.after }, v: { missing: obs.found == false, name: state.v.name or (obs.at.name != state.baseline.name), file: state.v.file or (obs.at.file != state.baseline.file), kind: state.v.kind or (obs.at.form != state.baseline.form), sig: state.v.sig or (obs.at.shape != state.baseline.sig), surface: state.v.surface or (obs.at.surface != state.baseline.surface), logic: state.v.logic or (obs.facts.body != state.baseline.body), place: state.v.place or (obs.at.after != state.baseline.after) }, status: \"settled\" }",
    ];

    fn vocabulary() -> BTreeSet<&'static str> {
        let mut out = BTreeSet::from([SETTLED, ABSENT, FLUX, RETURNED, REPLACED]);
        for shape in ALL {
            out.insert(shape.name);
            out.extend(shape.dims.iter().flat_map(|d| [d.name, d.status]));
        }
        out
    }

    fn step_at(
        rules: &[(String, String)],
        obs: &Value,
        state: &Value,
        taken_at: i64,
        entered_at: i64,
    ) -> Value {
        for (when, to) in rules {
            let guard = gmr_expr::parse(when)
                .unwrap_or_else(|e| panic!("guard `{when}` does not parse: {e}"));
            let ctx = Ctx::new(obs, state).at(taken_at, entered_at);
            match gmr_expr::eval(&guard, ctx) {
                Evaluated::Value(Value::Bool(true)) => {}
                Evaluated::Value(Value::Bool(false)) | Evaluated::Absent => continue,
                other => panic!("guard `{when}` is not a boolean: {other:?}"),
            }
            let body =
                gmr_expr::parse(to).unwrap_or_else(|e| panic!("body `{to}` does not parse: {e}"));
            return match gmr_expr::eval(&body, ctx) {
                Evaluated::Value(v @ Value::Object(_)) => v,
                other => panic!("new state of `{to}` is not an object: {other:?}"),
            };
        }
        panic!("no rule matched; a vector shape must end in a `true` rule");
    }

    fn step(rules: &[(String, String)], obs: &Value, state: &Value) -> Value {
        step_at(rules, obs, state, 0, 0)
    }

    fn contract() -> Vec<(String, String)> {
        rules(get("contract").unwrap())
    }

    fn dwell() -> Vec<(String, String)> {
        rules(get("contract-dwell").unwrap())
    }

    fn settled_state(shape: &Shape) -> Value {
        let mut obs = serde_json::Map::new();
        obs.insert("exact".into(), Value::Bool(true));
        obs.insert("found".into(), Value::Bool(true));
        let mut at = serde_json::Map::new();
        let mut facts = serde_json::Map::new();
        for d in shape.dims {
            let Reads::Since { obs: path, .. } = d.reads else {
                continue;
            };
            match path.split_once('.') {
                Some(("obs", rest)) => match rest.split_once('.') {
                    Some(("at", k)) => drop(at.insert(k.into(), "x".into())),
                    Some(("facts", k)) => drop(facts.insert(k.into(), "x".into())),
                    _ => drop(obs.entry(rest).or_insert(Value::from(0))),
                },
                _ => panic!("`{path}` is not an obs path"),
            }
        }
        obs.insert("at".into(), Value::Object(at));
        obs.insert("facts".into(), Value::Object(facts));
        let obs = Value::Object(obs);
        let rules = rules(shape);
        let opened = step(&rules, &obs, &Value::Null);
        step(&rules, &obs, &opened)
    }

    #[test]
    fn the_base_tables_are_the_ones_the_journal_already_runs() {
        assert_eq!(rules_of(get("contract").unwrap()), SHIPPED_CONTRACT);
    }

    #[test]
    fn every_generated_rule_is_a_program_the_evaluator_accepts() {
        for shape in ALL {
            for (when, to) in rules(shape) {
                gmr_expr::parse(&when)
                    .unwrap_or_else(|e| panic!("`{}`: guard `{when}`: {e}", shape.name));
                gmr_expr::parse(&to)
                    .unwrap_or_else(|e| panic!("`{}`: body `{to}`: {e}", shape.name));
            }
        }
    }

    #[test]
    fn the_contract_table_names_every_status_once() {
        let rules = rules_of(get("contract").unwrap());
        let statuses = [
            "absent",
            "missing",
            "kind-changed",
            "signature-changed",
            "surface-changed",
            "logic-changed",
            "moved",
            "renamed",
            "relocated",
            "settled",
        ];
        assert_eq!(rules.len(), statuses.len() + 1);
        for s in statuses {
            assert!(
                rules.iter().any(|r| r.contains(&format!("\"{s}\""))),
                "no rule can ever produce `{s}`"
            );
        }
    }

    #[test]
    fn a_shape_is_recognised_from_its_own_table() {
        for shape in ALL {
            assert_eq!(name_of(&transitions_of(shape)), Some(shape.name));
        }
    }

    #[test]
    fn state_carries_exactly_what_some_guard_compares_and_nothing_else() {
        for shape in ALL {
            let state = settled_state(shape);
            let top: BTreeSet<&str> = state
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect();
            let expected = match shape.dwells() {
                false => BTreeSet::from(["position", "baseline", "now", "v", "status"]),
                true => BTreeSet::from([
                    "position",
                    "baseline",
                    "now",
                    "v",
                    "status",
                    "phase",
                    "moved_count",
                    "last_moved_at",
                    "last_present",
                ]),
            };
            assert_eq!(
                top, expected,
                "`{}`: a field no guard reads still makes the state unequal, and \
                 `should_still` compares the whole state — so it writes a Transition \
                 with no bit lit. Facts that only inform a reader belong in the \
                 observation, which the journal already keeps",
                shape.name
            );

            let since: BTreeSet<&str> = shape
                .dims
                .iter()
                .filter_map(|d| match d.reads {
                    Reads::Since { field, .. } => Some(field),
                    Reads::Now { .. } => None,
                })
                .collect();
            for side in ["baseline", "now"] {
                let got: BTreeSet<&str> = state[side]
                    .as_object()
                    .unwrap_or_else(|| panic!("`{}`: {side} is not an object", shape.name))
                    .keys()
                    .map(String::as_str)
                    .collect();
                assert_eq!(got, since, "`{}`: {side}", shape.name);
            }

            let bits: BTreeSet<&str> = state["v"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect();
            assert_eq!(
                bits,
                shape.dims.iter().map(|d| d.name).collect::<BTreeSet<_>>(),
                "`{}`: one bit per dimension, no more and no fewer",
                shape.name
            );
        }
    }

    #[test]
    fn nothing_is_both_retired_and_shipping() {
        let live = vocabulary();
        for word in RETIRED {
            assert!(
                !live.contains(word),
                "`{word}` is on the tombstone list and also in this build's vocabulary; \
                 a note naming it would be reported as stale while being correct"
            );
        }
    }

    #[test]
    fn an_unknown_shape_names_what_this_build_ships() {
        let e = get("nope").unwrap_err();
        assert!(e.contains("roster") && e.contains("contract-dwell"), "{e}");
    }

    #[test]
    fn a_dwell_shape_watches_the_same_axes_as_its_base() {
        for shape in ALL.iter().filter(|s| s.dwells()) {
            let base = get(shape.name.trim_end_matches("-dwell")).unwrap();
            assert_eq!(axes_of(shape), axes_of(base), "{}", shape.name);
            assert_eq!(watch_of(shape), watch_of(base), "{}", shape.name);
        }
    }

    struct Shot {
        shape: &'static str,
        axis: &'static str,
        moves: &'static [&'static str],
        probe: &'static str,
        file: &'static str,
        pos: &'static str,
        before: &'static str,
        after: &'static str,
        elsewhere: Option<(&'static str, &'static str)>,
    }

    const AST: &str = "ast-map";
    const PROSE: &str = "prose-map";

    const RANGE: &[Shot] = &[
        Shot {
            shape: "contract",
            axis: "sig",
            moves: &["sig"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn f(x: u64) -> u64 { x }",
            after: "pub fn f(x: u64, y: u64) -> u64 { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "sig",
            moves: &["sig"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn f(x: u64) -> u64 { x }",
            after: "pub fn f(x: u64) -> u32 { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "sig",
            moves: &["sig"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn f(x: u64) -> u64 { x }",
            after: "pub async fn f(x: u64) -> u64 { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "sig",
            moves: &["sig"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn f(x: u64) -> u64 { x }",
            after: "pub unsafe fn f(x: u64) -> u64 { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "sig",
            moves: &["sig"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn f<T>(x: T) -> T { x }",
            after: "pub fn f<T: Clone>(x: T) -> T { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "surface",
            moves: &["surface"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn f(x: u64) -> u64 { x }",
            after: "fn f(x: u64) -> u64 { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "logic",
            moves: &["logic"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn f(x: u64) -> u64 { helper(x) }",
            after: "pub fn f(x: u64) -> u64 { other(x) }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "place",
            moves: &["place"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn a() {}\npub fn f(x: u64) -> u64 { x }",
            after: "pub fn a() {}\npub fn b() {}\npub fn f(x: u64) -> u64 { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "surface",
            moves: &["surface"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn f(x: u64) -> u64 { x }",
            after: "#[deprecated]\npub fn f(x: u64) -> u64 { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "name",
            moves: &["name"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f", "shape": "parameters parameter x u64 u64"}"#,
            before: "pub fn f(x: u64) -> u64 { x }",
            after: "pub fn g(x: u64) -> u64 { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "file",
            moves: &["file"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f", "shape": "parameters parameter x u64 u64"}"#,
            before: "pub fn f(x: u64) -> u64 { x }",
            after: "pub fn keep() {}",
            elsewhere: Some(("b.rs", "pub fn f(x: u64) -> u64 { x }")),
        },
        Shot {
            shape: "contract",
            axis: "missing",
            moves: &["missing"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "f"}"#,
            before: "pub fn f(x: u64) -> u64 { x }",
            after: "pub fn gone(x: u64) -> u64 { x }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "sig",
            moves: &["sig"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "X"}"#,
            before: "pub struct X { pub a: u64 }",
            after: "pub struct X { pub a: u64, pub b: u8 }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "sig",
            moves: &["sig"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "X"}"#,
            before: "pub struct X { pub a: u64 }",
            after: "pub struct X { pub a: u32 }",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "logic",
            moves: &["logic"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "X"}"#,
            before: "pub trait X { fn go(&self) -> u8 { 1 } }",
            after: "pub trait X { fn go(&self) -> u8 { 2 } }",
            elsewhere: None,
        },
        Shot {
            shape: "roster",
            axis: "missing",
            moves: &["missing"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "kind": "function"}"#,
            before: "pub fn a() {}",
            after: "pub struct A;",
            elsewhere: None,
        },
        Shot {
            shape: "roster",
            axis: "grew",
            moves: &["grew", "roll"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "kind": "function"}"#,
            before: "pub fn a() {}",
            after: "pub fn a() {}\npub fn b() {}",
            elsewhere: None,
        },
        Shot {
            shape: "roster",
            axis: "shrank",
            moves: &["shrank", "roll"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "kind": "function"}"#,
            before: "pub fn a() {}\npub fn b() {}",
            after: "pub fn a() {}",
            elsewhere: None,
        },
        Shot {
            shape: "roster",
            axis: "roll",
            moves: &["roll"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "kind": "function"}"#,
            before: "pub fn a() {}",
            after: "pub fn b() {}",
            elsewhere: None,
        },
        Shot {
            shape: "fingerprint",
            axis: "missing",
            moves: &["missing"],
            probe: PROSE,
            file: "a.md",
            pos: r#"{"file": "a.md", "heading": "H"}"#,
            before: "# H\n\nbody\n",
            after: "# Other\n\nbody\n",
            elsewhere: None,
        },
        Shot {
            shape: "fingerprint",
            axis: "drift",
            moves: &["drift"],
            probe: PROSE,
            file: "a.md",
            pos: r#"{"file": "a.md", "heading": "H"}"#,
            before: "# H\n\nbody\n",
            after: "# H\n\nrewritten\n",
            elsewhere: None,
        },
        Shot {
            shape: "contract",
            axis: "kind",
            moves: &["kind", "sig"],
            probe: AST,
            file: "a.rs",
            pos: r#"{"file": "a.rs", "name": "X"}"#,
            before: "pub struct X { pub a: u64 }",
            after: "pub enum X { A }",
            elsewhere: None,
        },
    ];

    fn fired(shot: &Shot, at: usize, shape: &Shape) -> Vec<String> {
        let dir = std::env::temp_dir().join(format!("gmr-range-{}-{at}", shape.name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(shot.file);
        let reg = gmr_coding_pack::registry_uncached();
        let probe = &reg
            .get(&gmr_core::ProbeName::new(shot.probe))
            .unwrap_or_else(|| panic!("`{}` is not linked in", shot.probe))
            .extract;
        let pos: Value = serde_json::from_str(shot.pos).unwrap();
        let look = |src: &str, extra: Option<(&str, &str)>| {
            std::fs::write(&file, src).unwrap();
            if let Some((name, body)) = extra {
                std::fs::write(dir.join(name), body).unwrap();
            }
            probe(&gmr_transport::inproc::Reach {
                cwd: dir.clone(),
                position: pos.clone(),
                params: serde_json::json!({}),
                budget: gmr_transport::inproc::Budget::within(
                    std::time::Duration::from_secs(30),
                    1 << 20,
                ),
            })
            .unwrap()
        };

        let rules = rules(shape);
        let opened = step(&rules, &look(shot.before, None), &Value::Null);
        assert!(
            set(&opened).is_empty(),
            "shot {at} did not open clean: {opened}"
        );
        set(&step(&rules, &look(shot.after, shot.elsewhere), &opened))
    }

    #[test]
    fn what_an_axis_answers_decides_when_its_bit_falls() {
        for d in get("contract").unwrap().dims {
            let carries = bit(d).contains(&format!("state.v.{}", d.name));
            match d.reads {
                Reads::Now { .. } => assert!(
                    !carries,
                    "`{}` answers about now, so carrying its own last bit would \
                     keep it up after the condition stopped holding",
                    d.name
                ),
                Reads::Since { .. } => assert!(
                    carries,
                    "`{}` answers about drift since you confirmed, so dropping \
                     its own last bit hands the signal to whoever observed first",
                    d.name
                ),
            }
        }
    }

    struct Beam {
        shape: &'static str,
        axis: &'static str,
        moves: &'static [&'static str],
        before: &'static str,
        after: &'static str,
    }

    const BEAMS: &[Beam] = &[Beam {
        shape: "value",
        axis: "value",
        moves: &["value"],
        before: r#"{"value": "1.2.3"}"#,
        after: r#"{"value": "1.3.0"}"#,
    }];

    fn lit(beam: &Beam, shape: &Shape) -> Vec<String> {
        let rules = rules(shape);
        let read = |text: &str| serde_json::from_str::<Value>(text).unwrap();
        let opened = step(&rules, &read(beam.before), &Value::Null);
        assert!(set(&opened).is_empty(), "beam did not open clean: {opened}");
        set(&step(&rules, &read(beam.after), &opened))
    }

    #[test]
    fn every_axis_can_be_moved_and_moves_alone() {
        for (at, shot) in RANGE.iter().enumerate() {
            for shape in [
                get(shot.shape).unwrap(),
                get(&format!("{}-dwell", shot.shape)).unwrap(),
            ] {
                assert_eq!(
                    fired(shot, at, shape),
                    shot.moves,
                    "shot {at} on `{}`: `{}` -> `{}`",
                    shape.name,
                    shot.before,
                    shot.after
                );
            }
        }
        for beam in BEAMS {
            for shape in [
                get(beam.shape).unwrap(),
                get(&format!("{}-dwell", beam.shape)).unwrap(),
            ] {
                assert_eq!(
                    lit(beam, shape),
                    beam.moves,
                    "beam `{}`/`{}`: `{}` -> `{}`",
                    shape.name,
                    beam.axis,
                    beam.before,
                    beam.after
                );
            }
        }
    }

    #[test]
    fn no_axis_of_any_shape_is_left_off_the_range() {
        for shape in ALL {
            let base = shape.name.trim_end_matches("-dwell");
            for axis in axes_of(shape) {
                assert!(
                    RANGE.iter().any(|s| s.shape == base && s.axis == axis)
                        || BEAMS.iter().any(|b| b.shape == base && b.axis == axis),
                    "`{}`'s `{axis}` has no shot; an axis nothing is known to move \
                     is how a dead one hides",
                    shape.name
                );
            }
        }
    }

    fn sighted(sig: &str, body: &str, file: &str, line: i64) -> Value {
        serde_json::json!({
            "schema": COORD_SCHEMA, "extractor": "ast-map", "found": true,
            "matched": ["file", "name"], "missed": [],
            "at": {
                "file": file, "kind": "function", "form": "function_item",
                "vis": "pub", "surface": "pub", "after": "", "name": "f", "shape": sig,
            },
            "facts": { "body": body, "line": line },
            "candidates": 1, "matches": [], "exact": true,
        })
    }

    fn unsighted() -> Value {
        serde_json::json!({
            "schema": COORD_SCHEMA, "extractor": "ast-map", "found": false,
            "matched": [], "missed": ["file", "name"],
            "at": Value::Null, "facts": Value::Null,
            "candidates": 0, "matches": [], "exact": false,
        })
    }

    fn bits(state: &Value) -> Vec<(String, bool)> {
        state["v"]
            .as_object()
            .expect("a vector shape always writes v")
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.as_bool()
                        .unwrap_or_else(|| panic!("v.{k} is not a bool: {v}")),
                )
            })
            .collect()
    }

    fn set(state: &Value) -> Vec<String> {
        bits(state)
            .into_iter()
            .filter(|(_, on)| *on)
            .map(|(k, _)| k)
            .collect()
    }

    #[test]
    fn three_changes_at_once_all_land() {
        let r = contract();
        let s = step(&r, &sighted("(a) -> B", "body1", "a.rs", 1), &Value::Null);
        assert_eq!(s["status"], "settled");
        assert!(set(&s).is_empty(), "a first sighting drifts from nothing");

        let s = step(&r, &sighted("(a, b) -> B", "body2", "a.rs", 9), &s);
        assert_eq!(set(&s), ["sig", "logic"]);
        assert_eq!(s["status"], "signature-changed");
    }

    fn tweak(mut sighting: Value, key: &str, to: &str) -> Value {
        sighting["at"][key] = Value::String(to.to_owned());
        sighting
    }

    #[test]
    fn narrowing_the_public_surface_is_its_own_axis() {
        let r = contract();
        let seen = || sighted("(a) -> B", "body1", "a.rs", 1);
        let s = step(&r, &seen(), &Value::Null);
        let s = step(&r, &tweak(seen(), "surface", ""), &s);
        assert_eq!(
            set(&s),
            ["surface"],
            "pub -> private moves nothing else, and used to move nothing at all"
        );
        assert_eq!(s["status"], "surface-changed");
    }

    #[test]
    fn a_struct_that_becomes_an_enum_is_not_a_changed_implementation() {
        let r = contract();
        let was = tweak(sighted("a: u64", "", "a.rs", 1), "form", "struct_item");
        let now = tweak(sighted("A; B", "", "a.rs", 1), "form", "enum_item");
        let s = step(&r, &was, &Value::Null);
        let s = step(&r, &now, &s);
        assert_eq!(s["status"], "kind-changed");
        assert_eq!(set(&s), ["kind", "sig"]);
    }

    #[test]
    fn every_axis_hands_the_memory_back_unless_the_note_narrows_it() {
        let shape = get("contract").unwrap();
        let watch = watch_of(shape);
        assert_eq!(
            watch,
            axes_of(shape),
            "an axis worth accumulating is an axis worth reporting; a note that \
             wants less says so itself"
        );
    }

    #[test]
    fn the_vector_is_ordered_by_priority() {
        let s = step(
            &contract(),
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
        );
        let axes: Vec<String> = bits(&s).into_iter().map(|(k, _)| k).collect();
        assert_eq!(
            axes,
            [
                "missing", "name", "file", "kind", "sig", "surface", "logic", "place"
            ]
        );
    }

    #[test]
    fn a_bit_never_clears_on_its_own() {
        let r = contract();
        let s = step(&r, &sighted("(a) -> B", "body1", "a.rs", 1), &Value::Null);
        let s = step(&r, &sighted("(a, b) -> B", "body1", "a.rs", 1), &s);
        assert_eq!(set(&s), ["sig"]);

        let s = step(&r, &sighted("(a, b) -> B", "body2", "a.rs", 1), &s);
        assert_eq!(set(&s), ["sig", "logic"], "sig must not clear itself");

        let s = step(&r, &sighted("(a, b) -> B", "body2", "a.rs", 1), &s);
        assert_eq!(
            set(&s),
            ["sig", "logic"],
            "a quiet observation clears nothing"
        );
    }

    #[test]
    fn a_miss_does_not_poison_the_other_axes() {
        let r = contract();
        let s = step(&r, &sighted("(a) -> B", "body1", "a.rs", 1), &Value::Null);
        let s = step(&r, &unsighted(), &s);
        assert_eq!(s["status"], "missing");
        assert_eq!(
            set(&s),
            ["missing"],
            "a miss says nothing about the other axes"
        );
    }

    #[test]
    fn a_miss_heals_itself_and_carries_real_drift_across() {
        let r = contract();
        let s = step(&r, &sighted("(a) -> B", "body1", "a.rs", 1), &Value::Null);
        let s = step(&r, &sighted("(a, b) -> B", "body1", "a.rs", 1), &s);
        let s = step(&r, &unsighted(), &s);
        assert_eq!(set(&s), ["missing", "sig"]);

        let s = step(&r, &sighted("(a, b) -> B", "body1", "a.rs", 1), &s);
        assert_eq!(set(&s), ["sig"], "missing heals; sig stays accumulated");
    }

    #[test]
    fn a_miss_before_any_baseline_still_captures_later() {
        let r = contract();
        let s = step(&r, &unsighted(), &Value::Null);
        assert_eq!(s["status"], "absent");
        assert!(
            s.get("baseline").is_none(),
            "writing a null baseline would strand this anchor in `absent`, got {s}"
        );

        let s = step(&r, &sighted("(a) -> B", "body1", "a.rs", 1), &s);
        assert_eq!(s["status"], "settled");
        assert_eq!(s["baseline"], s["now"]);
        assert!(
            set(&s).is_empty(),
            "the first real sighting is the baseline"
        );
    }

    #[test]
    fn a_near_miss_is_not_the_target() {
        let r = contract();
        let s = step(&r, &sighted("(a) -> B", "body1", "a.rs", 1), &Value::Null);

        let mut renamed = sighted("(z) -> Q", "otherbody", "a.rs", 40);
        renamed["exact"] = Value::Bool(false);
        renamed["found"] = Value::Bool(false);
        renamed["matched"] = serde_json::json!(["file"]);
        renamed["missed"] = serde_json::json!(["name"]);
        renamed["at"]["name"] = Value::String("g".into());

        let s = step(&r, &renamed, &s);
        assert_eq!(s["status"], "missing");
        assert_eq!(set(&s), ["missing"]);
        assert_eq!(
            s["now"]["sig"], "(a) -> B",
            "another object's reading must not become this anchor's"
        );
    }

    #[test]
    fn a_first_sighting_settles_rather_than_announcing_itself() {
        let r = contract();
        let first = step(&r, &sighted("(a) -> B", "body1", "a.rs", 1), &Value::Null);
        let again = step(&r, &sighted("(a) -> B", "body1", "a.rs", 1), &first);
        assert_eq!(
            first, again,
            "a distinct opening status would transition into settled on the very next \
             observation, handing back the memory for a change nobody made"
        );
    }

    fn fingerprint() -> Vec<(String, String)> {
        rules(get("fingerprint").unwrap())
    }

    fn section(heading: &str, print: &str, line: i64, exact: bool) -> Value {
        serde_json::json!({
            "schema": COORD_SCHEMA, "extractor": "prose-map", "found": exact,
            "matched": if exact { vec!["file", "heading"] } else { vec!["file"] },
            "missed": if exact { vec![] } else { vec!["heading"] },
            "at": { "file": "CLAUDE.md", "heading": heading, "fingerprint": print },
            "facts": { "line": line, "lines": 12 },
            "candidates": 1, "matches": [], "exact": exact,
        })
    }

    #[test]
    fn a_fingerprint_never_captures_a_section_it_did_not_actually_match() {
        let r = fingerprint();
        let fell_back = section("一、这十三条", "bac58fed", 7, false);

        let first = step(&r, &fell_back, &Value::Null);
        assert_eq!(first["status"], "absent", "a miss is not a baseline");
        assert!(
            first.get("baseline").is_none(),
            "nothing may be pinned from a fallback: {first}"
        );

        assert_eq!(
            step(&r, &fell_back, &first),
            first,
            "and it stays absent rather than settling into the wrong section"
        );
    }

    #[test]
    fn a_fingerprint_that_matched_captures_and_still_notices_the_heading_leaving() {
        let r = fingerprint();
        let captured = step(&r, &section("四、红牌", "aaa", 40, true), &Value::Null);
        assert_eq!(captured["status"], "settled");
        assert_eq!(captured["baseline"]["fingerprint"], "aaa");

        let after = step(
            &r,
            &section("一、这十三条", "bac58fed", 7, false),
            &captured,
        );
        assert_eq!(after["status"], "missing");
        assert_eq!(
            after["baseline"]["fingerprint"], "aaa",
            "the baseline survives; the fallback must not overwrite it"
        );
    }

    #[test]
    fn a_value_shape_opens_on_its_own_reading_not_on_a_found_flag() {
        for name in ["value", "value-dwell"] {
            let rules = rules_of(get(name).unwrap());
            assert!(
                !rules.iter().any(|r| r.contains("obs.found")),
                "`{name}`: the http probe answers 404 with `Outcome::NotFound`, which the base \
                 already reports as absent; asking for a `found` field as well would be the \
                 same fact recorded twice: {rules:?}"
            );
        }
        assert!(
            rules_of(get("value").unwrap())[0]
                .starts_with("not exists(state.baseline) and exists(obs.value) =>")
        );
    }

    #[test]
    fn deriving_the_opening_guard_left_every_shape_that_had_one_alone() {
        for name in ["contract", "roster", "fingerprint"] {
            assert!(
                rules_of(get(name).unwrap())[0]
                    .starts_with("not exists(state.baseline) and obs.found =>"),
                "`{name}` reports absence inside its facts and must keep doing so"
            );
        }
    }

    const WINDOW: i64 = 15 * 60;

    #[test]
    fn a_dwell_shape_opens_settled_with_its_counters_at_zero() {
        let r = dwell();
        let s = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
            10,
            10,
        );
        assert_eq!(s["status"], "settled");
        assert_eq!(s["phase"], "settled");
        assert_eq!(s["moved_count"], 0);
        assert_eq!(s["last_moved_at"], 10);
        assert_eq!(s["last_present"], s["now"]);
        let again = step_at(&r, &sighted("(a) -> B", "body1", "a.rs", 1), &s, 500, 10);
        assert_eq!(
            s, again,
            "a quiet observation must not rewrite the counters"
        );
    }

    #[test]
    fn a_change_enters_flux_and_stays_there_within_the_window() {
        let r = dwell();
        let s = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
            10,
            10,
        );
        let moved = step_at(&r, &sighted("(a, b) -> B", "body1", "a.rs", 1), &s, 100, 10);
        assert_eq!(moved["status"], "signature-changed");
        assert_eq!(moved["phase"], "flux");
        assert_eq!(moved["moved_count"], 1);
        assert_eq!(moved["last_moved_at"], 100);
        assert_eq!(set(&moved), ["sig"]);

        let quiet = step_at(
            &r,
            &sighted("(a, b) -> B", "body1", "a.rs", 1),
            &moved,
            100 + WINDOW - 1,
            100,
        );
        assert_eq!(
            quiet, moved,
            "inside the window nothing changes, so nothing is written"
        );
    }

    #[test]
    fn flux_settles_once_the_window_has_passed_without_a_change() {
        let r = dwell();
        let s = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
            10,
            10,
        );
        let moved = step_at(&r, &sighted("(a, b) -> B", "body1", "a.rs", 1), &s, 100, 10);
        let settled = step_at(
            &r,
            &sighted("(a, b) -> B", "body1", "a.rs", 1),
            &moved,
            100 + WINDOW,
            100,
        );
        assert_eq!(settled["phase"], "settled");
        assert_eq!(
            settled["status"], "signature-changed",
            "the axis stays reported until accepted"
        );
        assert_eq!(settled["moved_count"], 1);
        assert_eq!(settled["last_moved_at"], 100);
    }

    #[test]
    fn a_second_edit_inside_the_window_restarts_it() {
        let r = dwell();
        let s = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
            10,
            10,
        );
        let one = step_at(&r, &sighted("(a, b) -> B", "body1", "a.rs", 1), &s, 100, 10);
        let two = step_at(
            &r,
            &sighted("(a, b) -> B", "body2", "a.rs", 1),
            &one,
            200,
            100,
        );
        assert_eq!(two["phase"], "flux");
        assert_eq!(two["moved_count"], 2);
        assert_eq!(two["last_moved_at"], 200);
        assert_eq!(set(&two), ["sig", "logic"]);
    }

    #[test]
    fn moving_back_to_the_baseline_leaves_evidence() {
        let r = dwell();
        let s = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
            10,
            10,
        );
        let away = step_at(&r, &sighted("(a, b) -> B", "body1", "a.rs", 1), &s, 100, 10);
        let back = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &away,
            200,
            100,
        );
        assert_eq!(back["now"], s["now"], "the reading is the baseline again");
        assert_eq!(back["phase"], "flux");
        assert_eq!(
            back["moved_count"], 2,
            "a round trip is two movements, not none"
        );
        assert_eq!(
            set(&back),
            ["sig"],
            "the bit stays up until someone accepts"
        );
    }

    #[test]
    fn going_absent_counts_once_and_keeps_the_last_present_reading() {
        let r = dwell();
        let s = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
            10,
            10,
        );
        let gone = step_at(&r, &unsighted(), &s, 100, 10);
        assert_eq!(gone["status"], "missing");
        assert_eq!(gone["phase"], "absent");
        assert_eq!(gone["moved_count"], 1);
        assert_eq!(gone["last_present"], s["now"]);
        let still = step_at(&r, &unsighted(), &gone, 200, 100);
        assert_eq!(still, gone, "staying absent is not another movement");
    }

    #[test]
    fn a_return_with_the_same_signature_is_returned_not_new() {
        let r = dwell();
        let s = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
            10,
            10,
        );
        let gone = step_at(&r, &unsighted(), &s, 100, 10);
        let back = step_at(
            &r,
            &sighted("(a) -> B", "body9", "a.rs", 3),
            &gone,
            200,
            100,
        );
        assert_eq!(back["status"], "returned");
        assert_eq!(back["phase"], "returned");
        assert_eq!(back["moved_count"], 2);
        assert_eq!(
            set(&back),
            ["logic"],
            "the body did change while it was away"
        );

        let next = step_at(
            &r,
            &sighted("(a) -> B", "body9", "a.rs", 3),
            &back,
            300,
            200,
        );
        assert_eq!(next["status"], "logic-changed");
        assert_eq!(
            next["phase"], "settled",
            "returned lasts exactly one observation"
        );
    }

    #[test]
    fn a_return_with_another_signature_is_replaced() {
        let r = dwell();
        let s = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
            10,
            10,
        );
        let gone = step_at(&r, &unsighted(), &s, 100, 10);
        let other = step_at(
            &r,
            &sighted("(z) -> Q", "body1", "a.rs", 1),
            &gone,
            200,
            100,
        );
        assert_eq!(other["status"], "replaced");
        assert_eq!(other["phase"], "replaced");
        assert_eq!(set(&other), ["sig"]);
    }

    #[test]
    fn never_found_is_one_state_however_often_it_is_looked_for() {
        let r = dwell();
        let first = step_at(&r, &unsighted(), &Value::Null, 10, 10);
        assert_eq!(first["status"], "absent");
        assert_eq!(first["phase"], "absent");
        assert!(first.get("baseline").is_none());
        let again = step_at(&r, &unsighted(), &first, 500, 10);
        assert_eq!(again, first);
        let found = step_at(
            &r,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &again,
            600,
            10,
        );
        assert_eq!(found["status"], "settled");
        assert_eq!(found["moved_count"], 0);
    }

    #[test]
    fn an_anchor_switched_from_the_base_table_is_initialised_before_it_is_interpreted() {
        let base = contract();
        let r = dwell();
        let old = step_at(
            &base,
            &sighted("(a) -> B", "body1", "a.rs", 1),
            &Value::Null,
            10,
            10,
        );
        let old = step_at(
            &base,
            &sighted("(a, b) -> B", "body1", "a.rs", 1),
            &old,
            20,
            10,
        );
        assert!(old.get("moved_count").is_none());

        let init = step_at(
            &r,
            &sighted("(a, b) -> B", "body2", "a.rs", 1),
            &old,
            100,
            20,
        );
        assert_eq!(
            init["status"], old["status"],
            "initialisation interprets nothing"
        );
        assert_eq!(init["v"], old["v"]);
        assert_eq!(init["now"], old["now"]);
        assert_eq!(init["phase"], "settled");
        assert_eq!(init["moved_count"], 0);
        assert_eq!(init["last_moved_at"], 100);

        let then = step_at(
            &r,
            &sighted("(a, b) -> B", "body2", "a.rs", 1),
            &init,
            110,
            100,
        );
        assert_eq!(
            then["phase"], "flux",
            "the change waits one observation, no longer"
        );
        assert_eq!(set(&then), ["sig", "logic"]);
    }

    #[test]
    fn a_roster_returns_on_its_roll_and_a_fingerprint_on_its_print() {
        let roster = rules(get("roster-dwell").unwrap());
        let listing = |candidates: u64, roll: &str, found: bool| {
            serde_json::json!({
                "schema": COORD_SCHEMA, "found": found, "candidates": candidates,
                "roll": roll, "exact": found, "matched": [], "missed": [], "matches": [],
            })
        };
        let s = step_at(
            &roster,
            &listing(2, "function:a function:b", true),
            &Value::Null,
            10,
            10,
        );
        let gone = step_at(&roster, &listing(0, "", false), &s, 100, 10);
        assert_eq!(gone["phase"], "absent");
        let back = step_at(
            &roster,
            &listing(2, "function:a function:b", true),
            &gone,
            200,
            100,
        );
        assert_eq!(back["status"], "returned");
        let swapped = step_at(
            &roster,
            &listing(2, "function:a function:c", true),
            &gone,
            200,
            100,
        );
        assert_eq!(swapped["status"], "replaced");

        let print = rules(get("fingerprint-dwell").unwrap());
        let s = step_at(
            &print,
            &section("四、红牌", "aaa", 40, true),
            &Value::Null,
            10,
            10,
        );
        let gone = step_at(
            &print,
            &section("一、这十三条", "bac58fed", 7, false),
            &s,
            100,
            10,
        );
        assert_eq!(gone["status"], "missing");
        let back = step_at(
            &print,
            &section("四、红牌", "aaa", 41, true),
            &gone,
            200,
            100,
        );
        assert_eq!(back["status"], "returned");
        let rewritten = step_at(
            &print,
            &section("四、红牌", "bbb", 41, true),
            &gone,
            200,
            100,
        );
        assert_eq!(rewritten["status"], "replaced");
    }
}
