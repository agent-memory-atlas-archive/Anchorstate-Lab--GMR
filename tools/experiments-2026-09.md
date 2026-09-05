# Experiment record — findability, emergence, channel (2026-09-05)

Three experiments were run against this repository's own claims, on the
`fix/findable-ground` branch, each with a fresh `claude -p` agent per unit and
mechanical grading defined before the runs. **This document exists so that no
future agent re-runs them to learn what they already established.** The
harnesses stay under `tools/msis`, `tools/emergence` and `tools/channel` for
*new* questions; the section at the end says what would invalidate each verdict
and only then is a re-run worth its cost.

---

## 1. Findability acceptance (`tools/msis`)

**Question.** Can a fresh agent, holding only the gmr binary and SKILL.md,
assemble a task's minimal sufficient information set by walking the
anchor–memory network? Infrastructure bar (owner-set): *found, even if slowly*.

**Design.** Synthetic repo; ground truth = two notes joined by `rests-on`
across two anchors plus one journal-only conclusion citing both readings;
three distractor notes. The vendor token lives only in one note's body; the
conclusion lives only in the journal, so no file grep completes the set.
Gates: sufficiency (answer contains what only the full set yields) and
findability (`relied_on` ⊇ truth addresses). Minimality and edge-following are
reported metrics, never gates.

**Results.**

| scenario | branch | main baseline |
|---|---|---|
| s1 entry at rates coordinate | pass (precision 0.6) | pass (0.6) |
| s2 entry at checkout coordinate | pass (0.6) | — |
| s3 unannounced edit | pass (0.75, answered `reliable: false`) | pass (0.75) |
| s4 edge consumed, then `check` | **pass** (exit 1, 1 unclaimed) | **fail** (exit 0, silent) |

Every agent found all three ground-truth members including the journal-only
conclusion. s4 is mechanical and pins the level-triggered unclaimed fix; its
main-branch failure is the bug the branch repairs. That s1/s3 also pass on
main is the honest baseline: *found even if slowly* already held via
enumeration — the branch makes the signals trustworthy and the hops cheap
(`--carry`, `--reach`, `cobound.anchors`).

**Verdict.** Infrastructure bar met. Findability was never the scarce good on
a repo small enough to enumerate; signal correctness was.

---

## 2. Emergence (`tools/emergence`)

**Question.** Two modules with a real business coupling but **no path in the
code dependency graph at any depth** — does the memory layer grow the edge the
fact layer does not have, without anyone being told to connect them, and can a
later agent entering at A reach B through it?

**Design & validity.** `exporter` and `reporter` crates coupled only through
an implicit ledger-line format in `data/feed/`: zero imports, zero shared
types, zero shared path literals. Seed notes strictly module-local, sharing no
vocabulary; mechanical detector confirms `cross_edge: false` at seed. Stage-1
prompt gives only the *symptom* ("monthly totals ~100x too small") plus one
neutral sentence — "leave behind whatever a maintainer following that
discipline should leave behind" — with no link/connect/other-module wording
(prompt text archived in `tools/emergence/run.sh`). Run at two scales: 5
crates, and 15 crates including a strong decoy (`ingest`, parsing the
identical `id|amount|kind` shape from a different directory).

**Results — the edge emerges, 2/2, in three forms at once.** At both scales
the stage-1 agent left, unprompted: a new note on the reporter anchor whose
frontmatter `rests-on` the exporter note; `said` conclusions standing on
anchors of both modules; and an explicit `gmr bind` of the note onto the
exporter anchor, self-explained as *"so a change to the exporter's format
hands this note back too."* The scale-15 emergent note contains zero decoy
contamination and closes with the reason for the edge: *"If the exporter ever
changes its amount format again, this note is due."* The connection criterion
the agents used is **change coupling** — must-change-together — which the
dependency graph cannot carry.

**Stage 2 (enter at exporter only) and its controls.** Every arm found
reporter — at this repo size enumeration suffices, an honest ceiling — but
only the memory-laden arms carried *confirmation and inheritance*: they cited
the emergent note ("names this exact trigger", "due via rests-on link; its
text says so"), knew which standing conclusions the change would break and
that they should be retired, and classified the decoy precisely ("sibling
3-field format; diverges, not broken"). The seed-only control diagnosed its
own lack: *"Reporter dependency is undeclared in gmr … check will not hand
back the reconciliation note."* The haiku pair separated hardest: with the
memory edge, haiku inherited mechanisms it could not derive alone (the f64
ban, the two unit epochs, the decimal-point discriminant); without it, haiku
stayed at shallow generalities.

**Verdict.** The memory layer grows the missing edge on its own, for stated
reasons, and afterwards one `gmr read` at A carries any later agent to B with
confirmation the code graph cannot give.

---

## 3. Channel comparison (`tools/channel`)

**Question.** Minimality of a delivered set is unprovable (arg-min over all
sufficient sets is not computable). What *is* measurable: completeness of the
gmr channel, delivered volume, and Pareto dominance against enumeration.
Criteria pre-registered before any run.

**Design.** Same fixture and task (add an ISO currency field to the ledger
line). Arms: **A1** gmr-only (tool lockdown to `Bash(<gmr> *)`, verified via
`permission_denials`), **A2** enumeration-only (`.anchor/`, `memories/`,
`.claude/` and git history removed), **A3** both, **A1′** corpus probe
(`src/`, `data/`, `crates/` deleted — only journal + memories remain).
Models sonnet and haiku; scales 15 and 60 crates. Fact checklist: four shared
facts, one code-only fact (F2, trailing-field mechanics — only in reporter
source), one corpus-only fact (F6, standing conclusions lose ground — only in
the journal); contamination flag for the decoy. Metering: `--output-format
stream-json`, every tool result's bytes classified by source.

**Full matrix** (C = shared-fact completeness; ctm = decoy contamination):

| run | C | F2 | F6 | ctm | V bytes | turns |
|---|---|---|---|---|---|---|
| a1-15-sonnet r1 | 1.0 | ✓ | ✓ | – | 85,327 | 20 |
| a1-15-sonnet r2 | 1.0 | ✓ | ✓ | – | 79,714 | 14 |
| a1-15-sonnet r3 | 0.75 | ✓ | ✓ | – | 75,774 | 15 |
| a2-15-sonnet r1b (clean) | 0.25 | ✓ | – | ✗ | 11,891 | 17 |
| a2-15-sonnet r2 | 0.5 | – | – | ✗ | 16,930 | 35 |
| a2-15-sonnet r3c | 0.5 | ✓ | – | ✗ | 17,169 | 27 |
| a1p-15-sonnet (src-less) | 1.0 | ✓ | ✓ | –* | 121,859 | 27 |
| a3-15-sonnet | 0.75 | ✓ | ✓ | – | 117,062 | 30 |
| a1-60-sonnet r1 | 0.75 | ✓ | ✓ | – | 82,001 | 16 |
| a1-60-sonnet r2 | 1.0 | ✓ | – | – | 70,373 | 14 |
| a2-60-sonnet r1 | 0.5 | – | – | ✗ | 18,134 | 24 |
| a2-60-sonnet r2 | 0.5 | ✓ | – | ✗ | 15,258 | 19 |
| a1-15-haiku | 0.75 | ✓ | – | – | 45,563 | 12 |
| a2-15-haiku | 0.5 | – | – | ✗ | 22,674 | 28 |

\* A1′ mentions the decoy only as an inherited, correctly-qualified
classification ("affected only if the reader is tightened") — no substantive
contamination. The stored r1b row predates a grader fix; 0.25 is the
corrected score.

**Pre-registered criteria, judged honestly — two hold, two fail:**

1. **Completeness C(A1) ≥ C(A2): holds, decisively.** Median 1.0 vs 0.5,
   every replicate pair, both models, both scales. Enumeration misses F6 in
   7/7 runs (structurally unreachable) and usually F4 (the precision history —
   code cannot say *why*), and lists the decoy as affected in **7/7** runs;
   the gmr arm contaminates in **0/6**.
2. **Volume V(A1) < V(A2): fails, reversed ~4–5×.** Cause located: the JSON
   envelope. Each `read` returns full state/warrant/baseline structure and
   full-length hashes; agents never reach for `--lean`. The information is
   small; the packaging is not.
3. **Scale separation (V(A2) grows with repo size): fails, informatively.**
   Both arms' V stay flat 15→60 — enumeration is targeted grep, not
   exhaustive reading, so noise crates that share no vocabulary are skipped
   free. The separation lives in C, not V: enumeration's ceiling (~0.5, with
   contamination) does not move with scale either.
4. **A1′ corpus sufficiency: 1.0 — the strongest single data point.** With
   every source and data file deleted, journal + memories alone carried a
   full-marks answer. Note bodies total ≈2–3 KB inside ≈120 KB delivered:
   envelope-to-payload ≈ 20:1.

**Product conclusions.**

- **Envelope layering is the next highest-value change** (ahead of any
  retrieval/acceleration layer): agent-facing reads should default to an
  index-card level (address · claim line · warrant kind · links, short
  hashes), with bodies fetched by address. Storage stays verbatim — lossy
  compression at rest would break content addressing and version binding.
- **F6 at 0/7 vs 3/3 is the exclusivity proof for "memory as project
  contract"**: which judgments rest on what, and what a change breaks, cannot
  be rebuilt by reading code at any budget.
- The AI-written notes were the *densest* bytes measured, not the noise —
  low density lives in the envelope, not in free-form memory prose.

---

## Methodology incidents (all corrected before final numbers)

- A2's first run read deleted memories out of **git history**; fixed by
  rebuilding the arm's history, run excluded (kept in `results.jsonl` as
  `a2-15-sonnet-r1`).
- A grader short-circuit bug (`pop() and pop()`) inflated one A2 score;
  fixed, affected row re-scored offline.
- A session usage limit swallowed one batch mid-experiment; those cells were
  re-run after reset (`*-r1c` tags).

## Raw data

`tools/channel/results-2026-09.jsonl` (committed) holds the channel matrix.
Full stream transcripts, msis and emergence reports lived under
`.anchor/output/{msis,msis-main,emergence,emergence-big,emergence-haiku,channel}`
at run time (gitignored; copies under `/tmp` on the machine that ran them).
Prompts are verbatim in each harness's `run.sh`.

## When a re-run is worth it (and only then)

- Envelope layering ships → re-run **channel A1/A1′ only** (criterion 2
  should flip; expect V(A1) ≈ a few KB).
- The note protocol/lints change how notes are written → re-run **A1′** as
  the corpus-maturity gauge.
- The delivery or unclaimed semantics change → **msis s4** is the regression
  pin (mechanical, no LLM cost).
- Everything else here is settled for this task family; new questions deserve
  new fixtures, not repeats of these.
