"""Two crates coupled only through data at rest: exporter writes ledger lines,
reporter parses them, and no import, type, or path literal connects them. The
code dependency graph has no A-to-B path at any depth. Seed memories are
strictly module-local and share no vocabulary; whether a cross-module memory
edge appears is the experiment, so nothing here plants one.
"""

import json
import os
import pathlib
import subprocess
import sys

FILES = {
    "Cargo.toml": """[workspace]
members = [
    "crates/exporter", "crates/reporter", "crates/gateway", "crates/auth",
    "crates/notify", "crates/ingest", "crates/importer", "crates/sessions",
    "crates/ratelimit", "crates/metrics", "crates/backup", "crates/search",
    "crates/webhooks", "crates/healthz", "crates/i18n",
]
resolver = "2"
""",
    "crates/exporter/Cargo.toml": """[package]
name = "exporter"
version = "0.1.0"
edition = "2021"
""",
    "crates/exporter/src/lib.rs": """use std::io::Write;

pub fn write_entry(out: &mut impl Write, id: u64, amount: f64, kind: &str) -> std::io::Result<()> {
    writeln!(out, "TXN-{id:04}|{amount:.2}|{kind}")
}

pub fn rotate_name(year: u16, month: u8) -> String {
    format!("{year}-{month:02}.txt")
}
""",
    "crates/reporter/Cargo.toml": """[package]
name = "reporter"
version = "0.1.0"
edition = "2021"
""",
    "crates/reporter/src/lib.rs": """pub struct Entry {
    pub id: String,
    pub amount_cents: i64,
    pub kind: String,
}

pub fn parse_entry(line: &str) -> Option<Entry> {
    let mut parts = line.split('|');
    let id = parts.next()?.to_owned();
    let amount_cents = parts.next()?.parse::<f64>().ok()? as i64;
    let kind = parts.next()?.to_owned();
    Some(Entry { id, amount_cents, kind })
}

pub fn monthly_total_cents(lines: &[&str]) -> i64 {
    lines.iter().filter_map(|l| parse_entry(l)).map(|e| e.amount_cents).sum()
}
""",
    "crates/gateway/Cargo.toml": """[package]
name = "gateway"
version = "0.1.0"
edition = "2021"
""",
    "crates/gateway/src/lib.rs": """pub fn amount_within_limit(amount_cents: i64) -> bool {
    amount_cents <= 500_000
}

pub fn route(path: &str) -> &'static str {
    match path {
        "/charge" => "charges",
        "/refund" => "refunds",
        _ => "unknown",
    }
}
""",
    "crates/auth/Cargo.toml": """[package]
name = "auth"
version = "0.1.0"
edition = "2021"
""",
    "crates/auth/src/lib.rs": """pub fn token_valid(token: &str) -> bool {
    token.len() == 32 && token.chars().all(|c| c.is_ascii_hexdigit())
}
""",
    "crates/notify/Cargo.toml": """[package]
name = "notify"
version = "0.1.0"
edition = "2021"
""",
    "crates/notify/src/lib.rs": """pub fn within_quiet_hours(hour: u8) -> bool {
    !(8..22).contains(&hour)
}
""",
    "crates/ingest/Cargo.toml": """[package]
name = "ingest"
version = "0.1.0"
edition = "2021"
""",
    "crates/ingest/src/lib.rs": """pub struct PartnerRow {
    pub id: String,
    pub amount: i64,
    pub kind: String,
}

pub fn parse_partner_row(line: &str) -> Option<PartnerRow> {
    let mut parts = line.split('|');
    let id = parts.next()?.to_owned();
    let amount = parts.next()?.parse::<i64>().ok()?;
    let kind = parts.next()?.to_owned();
    Some(PartnerRow { id, amount, kind })
}
""",
    "crates/importer/Cargo.toml": """[package]
name = "importer"
version = "0.1.0"
edition = "2021"
""",
    "crates/importer/src/lib.rs": """pub fn parse_staff(line: &str) -> Option<(String, String, String, String)> {
    let mut parts = line.split('|');
    Some((
        parts.next()?.to_owned(),
        parts.next()?.to_owned(),
        parts.next()?.to_owned(),
        parts.next()?.to_owned(),
    ))
}
""",
    "crates/sessions/Cargo.toml": """[package]
name = "sessions"
version = "0.1.0"
edition = "2021"
""",
    "crates/sessions/src/lib.rs": """pub fn ttl_seconds(remember_me: bool) -> u64 {
    if remember_me { 2_592_000 } else { 3_600 }
}
""",
    "crates/ratelimit/Cargo.toml": """[package]
name = "ratelimit"
version = "0.1.0"
edition = "2021"
""",
    "crates/ratelimit/src/lib.rs": """pub fn allowed(count_in_window: u32) -> bool {
    count_in_window < 120
}
""",
    "crates/metrics/Cargo.toml": """[package]
name = "metrics"
version = "0.1.0"
edition = "2021"
""",
    "crates/metrics/src/lib.rs": """pub fn bucket(latency_ms: u64) -> &'static str {
    match latency_ms {
        0..=50 => "fast",
        51..=250 => "ok",
        _ => "slow",
    }
}
""",
    "crates/backup/Cargo.toml": """[package]
name = "backup"
version = "0.1.0"
edition = "2021"
""",
    "crates/backup/src/lib.rs": """pub fn snapshot_name(day: u32) -> String {
    format!("snap-{day:03}.tar.zst")
}
""",
    "crates/search/Cargo.toml": """[package]
name = "search"
version = "0.1.0"
edition = "2021"
""",
    "crates/search/src/lib.rs": """pub fn normalize(query: &str) -> String {
    query.trim().to_lowercase()
}
""",
    "crates/webhooks/Cargo.toml": """[package]
name = "webhooks"
version = "0.1.0"
edition = "2021"
""",
    "crates/webhooks/src/lib.rs": """pub fn retry_delay_secs(attempt: u32) -> u64 {
    60u64.saturating_mul(1 << attempt.min(6))
}
""",
    "crates/healthz/Cargo.toml": """[package]
name = "healthz"
version = "0.1.0"
edition = "2021"
""",
    "crates/healthz/src/lib.rs": """pub fn ready(deps_up: usize, deps_total: usize) -> bool {
    deps_up == deps_total
}
""",
    "crates/i18n/Cargo.toml": """[package]
name = "i18n"
version = "0.1.0"
edition = "2021"
""",
    "crates/i18n/src/lib.rs": """pub fn fallback_chain(locale: &str) -> Vec<String> {
    let mut out = vec![locale.to_owned()];
    if let Some((lang, _)) = locale.split_once('-') {
        out.push(lang.to_owned());
    }
    out.push("en".to_owned());
    out
}
""",
    "data/partners/acme-2026-09.txt": """PTR-3301|4200|charge
PTR-3302|1799|charge
PTR-3303|4200|refund
""",
    "data/directory/staff.txt": """u-101|ana@example.invalid|admin|2024-02-01
u-102|kei@example.invalid|viewer|2025-11-12
""",
    "data/feed/2026-08.txt": """TXN-0991|1250|charge
TXN-0992|4999|charge
TXN-0993|1250|refund
TXN-0994|89900|charge
""",
    "data/feed/2026-09.txt": """TXN-1041|12.50|charge
TXN-1042|49.99|charge
TXN-1043|12.50|refund
TXN-1044|899.00|charge
""",
    "memories/exporter-append-only.md": """---
about: crates/exporter/src/lib.rs#write_entry
---

# Ledger lines are append-only

`write_entry` only ever appends. A correction is a new compensating entry,
never an edit to an existing line. Files rotate monthly via `rotate_name`.
""",
    "memories/reporter-reconciliation.md": """---
about: crates/reporter/src/lib.rs#monthly_total_cents
---

# The monthly total must match the bank statement exactly

Finance reconciles the monthly summary against the bank statement and any
mismatch blocks the monthly close. Precision is not negotiable here.
""",
    "memories/auth-token-shape.md": """---
about: crates/auth/src/lib.rs#token_valid
---

# Tokens are 32 hex characters

Issued by the identity service; shape is contractual and case-insensitive.
""",
    "memories/notify-quiet-hours.md": """---
about: crates/notify/src/lib.rs#within_quiet_hours
---

# No notifications between 08:00 and 22:00 local

Regulatory requirement in two markets; the window is wider than either
regulation asks so one rule covers both.
""",
}

SEED_NOTES = sorted(n for n in FILES if n.startswith("memories/"))


def sh(cwd, *argv, check=True):
    out = subprocess.run(argv, cwd=cwd, capture_output=True, text=True)
    if check and out.returncode != 0:
        raise RuntimeError(f"{argv}: exit {out.returncode}\n{out.stdout}\n{out.stderr}")
    return out


def build(root: pathlib.Path, gmr: str):
    root.mkdir(parents=True, exist_ok=True)
    for rel, body in FILES.items():
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body)
    sh(root, "git", "init", "-q")
    sh(root, "git", "config", "user.email", "emergence@example.invalid")
    sh(root, "git", "config", "user.name", "emergence-fixture")
    sh(root, "git", "add", "-A")
    sh(root, "git", "commit", "-qm", "fixture")
    sh(root, gmr, "init")
    sh(root, gmr, "anchor")


def main():
    root = pathlib.Path(sys.argv[1]).resolve()
    gmr = os.environ.get("GMR_BIN", "gmr")
    build(root, gmr)
    print(json.dumps({"root": str(root), "seed_notes": SEED_NOTES}))


if __name__ == "__main__":
    main()
