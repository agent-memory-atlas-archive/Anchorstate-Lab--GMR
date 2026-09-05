---
about:
  - console/cli/src/verbs/read.rs#run
  - crates/gmr-runtime/src/read.rs#Instructions
watch: [sig, logic]
links:
  rests-on: [cli-read-vs-status, delivery-standing]
---

# The 2026-09 experiments are settled; the envelope is the next lever

Three experiments (`docs/experiments-2026-09.md`) established, with fresh
agents and pre-registered grading: the network's findability bar is met; a
cross-module memory edge emerges unprompted when an agent earns the causal
evidence, connected on change coupling; and the gmr channel dominates
enumeration on completeness (median 1.0 vs 0.5, decoy contamination 0/6 vs
7/7, the journal-only fact unreachable by any code reading) while **losing on
volume ~4-5x** — the JSON envelope of this read path, not the information,
carries the bytes. Src-less, journal+memories alone answered at full marks:
~2-3 KB of note bodies inside ~120 KB delivered, envelope-to-payload ~20:1.

Do not re-run these to learn what they already say. Re-run channel A1/A1'
only after the delivery envelope changes shape; msis s4 is the mechanical
regression pin for unclaimed semantics.

The product consequence lives at these coordinates: `read`'s agent-facing
default should approach an index-card level — address, claim line, warrant
kind, short hashes, bodies by address — with `--lean`-like behaviour the
default rather than the option nobody reaches for. Storage stays verbatim;
compression at rest would break content addressing and version binding.

## When this changes, ask

Did `run` or `Instructions` change what a default read delivers? Then the
20:1 ratio above is stale — re-measure with `tools/channel` (A1 and A1'
only) before citing it, and rewrite this note with the new number.
