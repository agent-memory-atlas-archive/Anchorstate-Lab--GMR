# Migration trial: sixteen notes against coding-v1

Question: do the slots in `coding-v1.yaml` hold what the existing notes carry? For each note: what fills a slot, what becomes an edge, what is dropped because it fills no slot, and what does not fit.

Verdict first. Fifteen of sixteen fit; one is retired. Three gaps were found and one is already folded into v1 (`review_question`). Roughly half of the notes' text fills no slot and is dropped: history, restatements of code, measurements, cross-references written as prose.

| note | Decision / Policy | edges | dropped | fit |
|---|---|---|---|---|
| addr-write_array | D: the depth counter must unwind on the error path, so the closure wrapper is load-bearing. about Function:write_array, write_object. review: does a new early return bypass `depth -= 1`? | — | "today it cannot be reached" is a reading | yes, already atomic |
| store-journal-guard | D: one shared guard for both backends. P: every write path passes through `guard`; the head compared is this anchor's own `MAX(seq)`, never the journal's. | verified_by → 2 tests; must_change_with(guard ↔ each backend's append) | "it used to check the token", "two branches lived here before" | yes; one sentence of that history survives as rationale |
| anchor-RunSettings | P: no field of RunSettings may be an input to the transition function or enter the log. Entry test = the review question. D: `cadence_secs: None` means deployment default. | must_change_with(RunSettings ↔ Anchor) | explanation of each field (reading) | yes |
| lib-narrow_of | D: the tree walked and the subtree reported are two axes. P: no call site joins `reach.cwd` with `params.root`. | rests_on → Function:under (survey) | "the seven layer anchors" (stale and a restatement of `layers`) | yes; the stale sentence disappears with the drop |
| memory-Binding | D: Binding holds only the idempotent relation; occasion fields live in BindingRecord. D: `saw` lives on the occasion, not the relation. P: `Claim::Stored` serializes as a bare `Ref`. | must_change_with(Binding ↔ BindingRecord); verified_by → 2 tests | narrative of how `saw` was placed | yes; one note became three Decisions |
| runtime-closure-is-structure | D: closure is sticky in `fold`; correcting a bad criterion means a new generation, never resurrection. | verified_by → 3 tests | how each test is constructed (reading) | yes; test names map straight onto verified_by |
| runtime-carry-linked | D: relevance beyond one hop is the domain's judgment, not the substrate's. Function:carry_linked.contract: one hop, opt-in, carried records marked not grounded, shared budget. | reads → Field:Instructions.carry; governed_by → P:content-budget | comparison with `reach` (belongs on Function:reaching), "that has not changed" | yes |
| runtime-warrant | D: Warrant is two enums, not one, because both halves are true at once. D: Finished is checked before Absent. P: the base ships no verdict fold. | decided_by from Type:Warrant, Holding, Knowledge | "briefly shipped Bearing", counts of memories, variant enumerations | yes; eight coordinates became three Decisions |
| cli-read-vs-status | D: `read` and `status` stay separate verbs. P: `read --json` prints addresses, not names. | rests_on → reading of Type:AnchorView (the note quotes its fields) | "a pass once considered", field lists, "the list said `attempts` long after…" | yes; more than half dropped |
| memories-lint | D: whether long-hand is warranted is decided by routing, not convention. P: a branch that judges false may only under-report. | — | mapping onto README's four reasons | yes as structure; the entity is retired in the target design |
| memories-foreign-words | P: a header this format cannot read is reported, never translated. | — | why `FRONTMATTER_WORDS` duplicates field names (reading) | same as above |
| layers | P per Module: what it may only hold (eight). review: the entry question per layer. | governed_by from eight Modules | "that claim was once false", "gmr-content was added late", axis definitions (reading of shapes) | yes; the anchor part becomes the Module's roster reading |
| constitution | P: CLAUDE.md §1 and §7 are owner criteria; a change is either the owner changing them or a violation. | governed_by from File:CLAUDE.md | prose-map breadcrumb mechanics (tool usage) | yes |
| three-layers | ontology-level: Decision/Policy and Claim are distinct kinds with distinct lifecycles. P: criteria for a Decision are never derived from Claims; a Claim is never promoted without a person. | — | "this note exists because the merge was nearly made" | yes, but as an axiom of this file, not as an instance |
| gmr-not-entailment | P: the runtime never compares a claim's text against a reading; judgment is the agent's. about Module:crates/gmr-runtime. | — | comparison with citation literature | yes |
| positioning | — | — | all of it | retired: it states a positioning the owner has replaced |

## What the drops were

Across the sixteen: history ("used to", "once", "the last time"), restatements of code the probe can read, measurements with a date, and cross-references written as sentences. None of it fills a slot. The cross-references become edges, which is the one drop that is a conversion rather than a loss.

## Gaps

1. **`review_question`.** Fourteen of sixteen notes end with "When this changes, ask". It is not rationale and not claim; it is what a person should reconsider when a governed entity goes stale. Added to Decision and Policy in v1.
2. **A note is a bundle.** One file routinely holds three to eight Decisions about different entities. Migration splits them; the file stops being an identity. The trial confirms the split is mechanical once slots exist.
3. **Entities the target design retires.** Notes about lints, CLI verbs and shapes describe machinery that the target design removes. Their Decisions fit the slots, and the entities they are about will be retired; the migration needs a `retired` standing so they are not carried forward as live.
4. **Ontology-level notes.** `three-layers` is not an instance of anything; it is a statement about the kinds. It belongs in this file as an axiom. `positioning` is the same kind of note and is simply superseded.

## What the trial did not test

Whether an agent, given `coding-v1.yaml` and one of these notes, produces the same split. That is the next trial and it costs one model call per note.
