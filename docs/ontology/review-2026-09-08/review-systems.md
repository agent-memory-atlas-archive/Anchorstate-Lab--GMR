# GMR review — adversarial systems architect / ecosystem lens

Legend: **(M)** measured today, **(C)** inferred from code, **(O)** opinion.

## 1. Survivor audit

Source LOC (M): crates 13,856 · console/cli 13,414 · SDKs 906 · batteries 7,881 · packs ~1,900 ≈ 37.5K, plus ~11.7K test LOC (858 tests).

| Mechanism | Verdict | Reason |
|---|---|---|
| Journal + fold (`gmr-core/journal.rs`, 909 LOC) | **shrink to a readings cache** | (M) 62,636 rows for 847 anchors; 46,777 (75%) are `still` = "nothing changed", 360 B each; a full `check` appends ~305 KB of nothing. Staleness needs one row per address: `(address, hash, at)`. `Entry::{Still,Attempt,Revise,Close}`, `AnchorState`, `fold/scan/resume` go; `addr.rs` and `A(outcome,d)` stay. |
| `gmr-expr` + rule tables (`shapes.rs` 1,261, `rules.rs`) | **delete** | The sole predicate is `stored ≠ current`. Axes (sig/logic/file/line) are a fixed diff classifier over the extract record, not a language. `depends` dies with `said`. ~3,100 LOC, 0% survives. |
| Warrant/Holding/Shown/Depends, `said`/`ground` (`read.rs` 1,375) | **delete, keep one bit** | (C) Holding 7 variants + Knowledge/Blind + Footing 8 collapse to `{current, stale, absent}` plus a transport error. `Shown` becomes a write-time check: `rests_on` must be an address the runtime handed out. (M) 13 self-attested claims exist. |
| Content providers (git/mem0/claude-code/declared/http/local_file, 1,708) | **shrink to `declared`** | Memory = net rows: nothing to fetch, no before/after. mem0/claude-code cannot enforce the slot rule, so they are `rests_on` targets, not sources. |
| `links` table | **delete; re-create as `relations`** | (M) 388 rows, 381 revoked, 7 live. Identity `(kind,from,to)` + provenance + reason; `source='unknown'` unrepresentable. |
| atlas (398 + 447 + JS), adopt | **delete** | Viewer and importer for the anchor–note graph the target retires. |
| `accept --why` sealing (`sealed`: 4,108 rows, 9.7 MB, max 108 KB) | **delete** | (M) it seals full state snapshots beside rationale. Rationale is a required slot on Decision/Policy, versioned by git; `supersedes` is the act. |
| doctor (45 JSON keys, ~25 buckets) | **delete ~22** | Buckets are failure modes of deleted mechanisms (`stranded`, `undeclared`, `unseen_*`, `chain_break`, `skill_stale`, note lints). Keep: stale count, entity without reading, dangling edge. |
| Extractors | **keep ast/prose/addr-map; delete name-map, survey index** | (M) `survey-index.sqlite` is 355 MB for 445 files: 362K posting rows, `callee` values holding whole file bodies (atlas.js ≈ 19 KB per row). The 12.7 MB file-hash `extract-cache.json` is the right shape. `path#name` keys make name-map moot. |
| sync / frontmatter bindings (`sync.rs` 1,242, `memories.rs` 1,118, `notes.rs` 395) | **delete** | The note stops being an identity (migration-trial gap 2); `write` is the only ingress. (M) 2,579 binding rows become `rests_on` edges. |
| `gmr-budget` (220) | **keep** | Every probe call needs a deadline. |
| transports (3,160) | **shrink to inproc + script** | http/sql have no user; script is how non-code facts arrive. |

Survival (O): core 30% · expr 0% · budget 100% · probe 80% · content 30% · store 20% · runtime 10% · cli 15% · batteries 35% · packs 70% ≈ 9–10K of 37.5K LOC; the target runtime is ~6–8K new.

## 2. Storage for three interlinked graphs

The worry is real, the diagnosis is wrong. (M) Local state is 473 MB (memory.db 105 + survey-index 355 + cache 13) for 847 anchors, 197 notes, 7 live links. Cross-link storage (`binding_anchors`) is 450 KB. Space goes to observation history (journal pages 87 MB; 11,089 transitions at 2.5 KB carry the full state twice, including a 331 B signature string) and a per-generation full-repo attribute inventory. The code did not cheat on links; it over-stored readings.

Three graphs are one property graph whose node kinds differ by **mutability and home**, not by table:

| Kind | Home | Materialized | Recomputed |
|---|---|---|---|
| Entity + slots | git, `net/<source path>.yaml` — one net file per source file, so a PR touching `addr.rs` touches `net/…/addr.rs.yaml` | values, provenance | — |
| Relation (`(kind,from,to)` unique, stored from-side) | git, same file as `from` | kind, to, provenance, reason | reverse index (local) |
| Claim | git, `net/claims/<task>.yaml` | text, `rests_on` address | delivery eligibility |
| Reading | **local only** | `(address, hash, at)` behind a file-hash gate | always, from the repo |
| Locate index (FTS, name segments, optional embeddings) | local only | — | from net + repo |

A clone carries the net and nothing else; the local db is a deletable cache. Structural edges are readings (§4).

Cost (O; entity ≈ 200 B, edge ≈ 120 B):

- 1,000 entities / 5,000 edges: ~0.8 MB YAML in ~300 files; local index ~1.5 MB. `enter(modify, F)` reads deg(F) ≈ 5 × ~8 closure lines ≈ 40 rows ≈ 6 KB, versus (M) 18.5 KB today for one coordinate carrying 3 KB of memory text.
- 50,000 / 250,000: ~40 MB text in ~5,000 files (fine for git; one file would not be); local sqlite ~100 MB. `enter` cost is local degree, unchanged. `check` after a commit touching n files is O(n).

Never materialize the per-action closure; always materialize reverse edges, or `modify` is a scan.

## 3. Convergence

Attacks, each with today's analog:

1. **Nobody deletes wrong Claims.** Retire-on-stale only touches Claims on moved code. (M) 580/600 anchors settled; the `runtime-carry-linked` note was wrong from the day it was written and stayed green. Staleness is not wrongness; the target inherits this.
2. **Garbage is unbounded.** "Completeness grows with tasks" is monotone; Claims keyed `task seq` only add.
3. **Contradicting slot values.** v1 has `supersedes` for Policy/Decision/Claim, nothing for slots: two agents writing `Function.purpose` produce a git conflict or a silent last-write. `contradicts` is manual; an agent that never saw the other value cannot declare it.
4. **Hubs.** `must_change_with: any→any` and `rests_on: any` have no degree cap. (M) 38 notes are about `read.rs`; `enter(modify, read.rs#X)` reads every Claim resting on it. Accumulation is fastest where delivery is most expensive.
5. **Stale-marks-cleared** counts activity: maximized by restating every stale slot, or by `retire`. No oracle.

Minimal rules without workers or judges:

- **R1 slot identity:** one value per (entity, slot); write replaces; the old value goes to the journal.
- **R2 relation identity:** `(kind, from, to)` unique; re-write updates provenance.
- **R3 compare-and-swap:** a write carries the hash of the value it replaces; mismatch is refused. Writers serialize with a visible predecessor; "contradiction" becomes "you did not read the current value".
- **R4 caps:** `one_sentence` ≤ 200 B; `Claim.text` ≤ 300 B; ≤ 5 live Claims and ≤ 8 `must_change_with` per entity — the sixth write forces a retire.
- **R5 reason required** on `must_change_with`, `contradicts`, `rests_on: any`.
- **R6 read-as-vote:** a Claim not delivered-and-kept in K subsequent `enter`s on its entity leaves delivery (journal, not deleted). The only shrink mechanism.
- **R7 stale blocks done:** `enter` delivers stale items with the diff; that task's `write` is refused until each is re-attested or retired. The only clear mechanism.
- **R8 metric:** (a) of items delivered stale, the split re-attested / rewritten / retired; (b) bytes the agent read outside the delivered subgraph before its first write. If (b) does not fall over tasks, nothing is converging.

## 4. Fact-layer edges

(M) `packs/coding/extract/src/ast.rs:163` keys `call`/`import` candidates by callee **text**; there is no resolver (35,652 unresolved `callee` postings). "Isolated fact nodes" means calls are extracted as strings and never joined to definitions. (M) CodeGraph on this repo: 5,347 nodes, 19,904 edges (9,093 calls) and **26,813 unresolved refs** — the wholesale index is about half-blind on Rust.

| Option | Failure mode |
|---|---|
| Probe on demand, never stored | Per-`enter` parse of every possible caller → needs an index anyway; cross-repo callers impossible. |
| Agents write when observed | Task-shaped coverage; wrong (unresolved) names; stale on the next commit with nothing recomputing them. |
| Import CodeGraph | Forbidden dependency; foreign identity; re-import is wholesale replace with no provenance; half unresolved. |

Decision (O): structural edges are **readings** — computed by the pack from the file-hash cache, materialized locally, recomputed per changed file, never in git, never agent-written. `calls/imports/contains/implements` leave v1's `relations:` for a `readings:` block (`callers(x)`, `members(x)`). Add the missing name+scope resolver (name-map's data, reused) answering a candidate set when ambiguous. The net keeps only non-recomputable edges: `produces/consumes`, `governed_by`, `decided_by`, `must_change_with`, `rests_on`, `contradicts`. The owner's A/B case is a `must_change_with` with a reason — what agents earn.

## 5. Ecosystem and boundaries

- **Cross-repo:** the entity lives in its owner's repo; the edge lives from-side with a foreign key `repo@path#name`, `rests_on` a reading of a *published* artifact (schema file, package version, git sha) via the script transport — never a foreign working tree.
- **Non-code facts:** entities read by a script probe (a market, a registry) or `user-said` with no probe — stale only by `supersedes`. v1's provenance already carries this.
- **Multiple memory stores:** a store that cannot enforce slots is not a net source; it is a `rests_on` target.

Product boundary: (1) ontology format + validator; (2) net file format + write protocol (slots, CAS, provenance); (3) staleness contract = `A(outcome, d)` + probe contract; (4) a per-repo runtime for `locate/enter/check`. (2)+(3) ship alone: a repo can carry a net without running GMR.

**ATA** submits DAGs `evidence → sub_thesis → main_thesis` (C1–C8) with `EvidenceSource{name,url,published_at}` and `EvidenceMetric` — a Claim graph with `rests_on`. ATA fixes three surfaces and red-cards a fourth, so GMR is a library inside `submit`, not a service: ATA's own ontology (Instrument, Thesis, Evidence), a market-data script probe, `write` at submit computing `rests_on` per evidence, `check` at evaluation naming which readings moved. That needs a `gmr-net` crate with no IO — addressing and validation — not the CLI.

## 6. Implementation path

**Phase 0 — falsifier (this month, no code):** hand-write the net for one subsystem (the `read.rs` path, ~40 entities, its 38 notes). Run 10 real tasks twice: agent given a hand-serialized `enter(modify, X)` (≤ 4 KB) vs `gmr read --json` + SKILL.md. Measure bytes read outside the delivered set before first edit, test pass rate, and whether the end-of-task write is slot-valid unprompted. **Falsified if** the subgraph arm reads as much outside it as control, or < 50% of writes fill a slot without a judge.

**Phase 1 — net + write:** add `gmr-net` (format, validator, CAS write, local index). Delete sync/memories/notes, bindings, links, atlas, adopt, said/ground, expr, shapes, survey index. Outcome: `write` refuses slot-less text; 197 notes migrated with the drop ratio measured.

**Phase 2 — enter + check:** action closures, reverse index, readings cache replaces journal. Delete Warrant/Holding/Shown, sealed, ~22 doctor buckets. Outcome: `enter` p90 ≤ 4 KB and ≤ 50 ms; `check` after a commit is O(touched files).

**Phase 3 — locate + ecosystem:** hybrid locate, foreign keys, structural-edge resolver, embeddable crate. Outcome: ATA `submit` writes Claims with `rests_on`; `check` names moved evidence.
