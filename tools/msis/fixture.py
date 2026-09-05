"""Builds the synthetic repository the findability scenarios run against.

The ground-truth minimal sufficient information set for the discount task:
  A  memories/checkout-rounding.md   (bound to src/checkout.rs#total)
  B  memories/rates-floor.md         (bound to src/rates.rs#discount_bps, holds "Kestrel"/275)
  C  said:cart-total-baseline        (recorded on the checkout anchor, journal-only)
Distractors: memories/logging-style.md, memories/deploy-window.md,
             memories/rates-history.md (linked from B, irrelevant).
"""

import json
import os
import pathlib
import subprocess
import sys

FILES = {
    "src/checkout.rs": """pub fn total(cents: u64) -> u64 {
    cents - cents * crate::rates::discount_bps() / 10_000
}
""",
    "src/rates.rs": """pub fn discount_bps() -> u64 {
    250
}
""",
    "src/logging.rs": """pub fn log_line(key: &str, value: &str) -> String {
    format!("{key}={value}")
}
""",
    "src/audit.rs": """pub fn audit_stamp(id: u64) -> String {
    format!("audit-{id}")
}
""",
    "src/lib.rs": """pub mod audit;
pub mod checkout;
pub mod logging;
pub mod rates;
""",
    "memories/checkout-rounding.md": """---
about: src/checkout.rs#total
links:
  rests-on: [rates-floor]
---

# The discount never rounds up

`total` uses floor division deliberately: finance policy FLOOR-7 requires that a
discount is never rounded in the customer's favour. The computation assumes the
basis-point denominator fixed by [[rates-floor]]; change either side without the
other and invoices drift by a cent per order.
""",
    "memories/rates-floor.md": """---
about: src/rates.rs#discount_bps
links:
  see-also: [rates-history]
---

# The discount rate is capped by contract

`discount_bps` is the sole source of the discount basis points. The pricing
contract with vendor Kestrel caps it at 275 bps; raising it past that breaches
the contract regardless of what the code allows. The denominator is fixed at
10_000 — [[checkout-rounding]] explains why the floor on that division matters.
""",
    "memories/logging-style.md": """---
about: src/logging.rs#log_line
---

# Logs are single-line key=value

One event per line, no JSON, so grep stays the query language.
""",
    "memories/deploy-window.md": """---
about: src/lib.rs
---

# Deploys happen on Tuesdays

The payment processor's change freeze covers every other weekday.
""",
    "memories/rates-history.md": """---
about: src/rates.rs
---

# Historical rates live in the finance archive

Superseded rate values are archived by finance; this repository only ever holds
the current one.
""",
}

TRUTH = {
    "records": ["git:memories/checkout-rounding.md", "git:memories/rates-floor.md"],
    "said": "said:cart-total-baseline",
    "distractors": [
        "git:memories/logging-style.md",
        "git:memories/deploy-window.md",
        "git:memories/rates-history.md",
    ],
    "token": "kestrel",
    "cap": "275",
    "new_total": "9700",
}


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
    sh(root, "git", "config", "user.email", "msis@example.invalid")
    sh(root, "git", "config", "user.name", "msis-fixture")
    sh(root, "git", "add", "-A")
    sh(root, "git", "commit", "-qm", "fixture")
    sh(root, gmr, "init")
    sh(root, gmr, "anchor")
    sh(root, gmr, "anchor", "src/audit.rs#audit_stamp")

    def address_of(coordinate):
        read = sh(root, gmr, "read", coordinate, "--json")
        views = json.loads(read.stdout)
        view = views[0] if isinstance(views, list) else views
        return view["fact_address"]

    sh(
        root,
        gmr,
        "said",
        "the standard 10_000-cent cart totals 9750 cents at the current discount rate",
        "--on",
        "src/checkout.rs#total",
        "--on",
        "src/rates.rs#discount_bps",
        "--saw",
        address_of("src/checkout.rs#total"),
        "--saw",
        address_of("src/rates.rs#discount_bps"),
        "--depends",
        "all(anchors, not state.v.logic)",
        "--id",
        "cart-total-baseline",
    )


def mutate_rates(root: pathlib.Path, gmr: str):
    rates = root / "src/rates.rs"
    rates.write_text(rates.read_text().replace("250", "260"))
    sh(root, gmr, "observe", check=False)


def mutate_audit(root: pathlib.Path, gmr: str):
    audit = root / "src/audit.rs"
    audit.write_text(audit.read_text().replace('"audit-{id}"', '"stamp-{id}-v2"'))
    sh(root, gmr, "observe", check=False)


def main():
    root = pathlib.Path(sys.argv[1]).resolve()
    gmr = os.environ.get("GMR_BIN", "gmr")
    mutation = sys.argv[2] if len(sys.argv) > 2 else None
    build(root, gmr)
    if mutation == "s3":
        mutate_rates(root, gmr)
    if mutation == "s4":
        mutate_audit(root, gmr)
    print(json.dumps({"root": str(root), "truth": TRUTH}))


if __name__ == "__main__":
    main()
