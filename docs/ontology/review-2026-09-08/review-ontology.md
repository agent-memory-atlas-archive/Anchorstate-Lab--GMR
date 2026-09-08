# GMR review — knowledge-representation and ontology lens

Tags: **[M]** measured today, **[C]** read from code, **[O]** opinion.

## 1. Is fact=KG / anchor=state-machine graph / memory=ontology the right decomposition?

No. It confuses schema with instances and puts the graph in the wrong layer.

**"Memory layer = ontology" is a category error.** An ontology is a TBox: types, relation types, constraints, closure rules. Memories are ABox instances filling its slots. `coding-v1.yaml` already says this itself (line 2). The ontology is one small human file; memories are thousands of assertions.

**"Fact layer = knowledge graph" conflates two ABoxes.** What a probe reads (`calls`, `contains`, signature, body hash) is an *observed* ABox: recomputable, no author, closed-world per reading. What an agent or person writes (`purpose`, `decided_by`, a Policy) is an *asserted* ABox: authored, open-world, needs provenance. `.codegraph/` already materialises the first; v1 lists `calls/imports/contains/implements/reads/exposes` as writable relations (lines 107–114) while line 179 says probe-readable things are never memory and line 141 says `callers(self)` is "never stored as edges". Three answers to one question **[C]**.

**"Anchor = state machine" is a justification link wearing a costume.** [C] `anchor.rs:153-161` holds key, probe, transitions, terminal. The rule table is not hand-written per coordinate: `shapes.rs:86-101` generates it from one of four shapes (`ALL`, line 131), and 847 anchors carry the same generated text. What the machine computes is a sticky vector `v` of eight "axis moved since the baseline a human accepted" bits (§4.5: level-triggered, cleared only by `accept`). That is exactly a JTMS node: the memory is IN while its justifying readings are unchanged; when a fact_address it rests on changes, the label becomes OUT-pending and a human relabels. §3.10's "memory drifts / inference loses ground" is Doyle's distinction between a premise (needs a person to retract) and a derived node (retracts when support is lost). The named states (`settled`, `renamed`, …) are only the argmax over `v`; §4.5 itself says `status` is derived.

[M] The costume costs 51% of every read: 9,428 of 18,580 bytes of `gmr read carry_linked --json` are the rule table, byte-identical for every `contract`-shape anchor.

What the machine does that a TMS does not: **entity resolution across renames** — `packs/coding/extract/DESIGN.md:81-93` narrows `file → kind → name → shape` and reports what missed. That is an identity service for KG keys; it survives.

**Correct decomposition [O]:**

| layer | what it is | survives from today |
|---|---|---|
| TBox | `coding-v1.yaml` — types, relation types with `source: observed\|asserted`, action signatures, completeness rules | new |
| Observed ABox | graph a probe emits: structural relations + fact_addresses; recomputed, never agent-written | probes, fact_address, `.codegraph/` |
| Asserted ABox | slot-fills and asserted relations, each a nanopub (assertion + provenance + publication) | memories, Said, Decision/Policy — merged into one store, distinguished by `author` and `source` |
| Identity | entity ids stable across rename/move/sig-change, resolved by probe | the extractor's resolver, `position` |
| Labelling | derived IN/OUT per assertion: OUT iff any cited fact_address changed; human `accept` re-labels premises | `v`, `warrant`, `holding`, `accept --why` |

Anchor as a persisted per-coordinate object dissolves into (identity, cited fact_address, label). The journal survives as PROV activity log. "Inference" (`Said`) merges into the asserted ABox with `author: agent` and no human acceptance — `Claim` in v1 is that already.

## 2. `coding-v1.yaml` as an ontology — top five defects

**D1. Identity keys are descriptions, not identifiers.** `Function: key: path#name(shape)` (line 31). A signature change — the most common event the system watches — changes the key, orphaning every slot and edge. `Type` is `path#name` (38), `Field` `path#Type.name` (44): three keying conventions in one file. KR rule: an IRI must survive the changes you intend to track. *Replace:* opaque `id` minted at first observation; `label: path#name` and `shape` become observed properties; the extractor's resolver keeps the id attached across rename/move/sig; provenance cites fact_address, which already carries shape. 

**D2. Relations lack `source`; observed and asserted are mixed.** Slots declare `source:` (23), relations do not (106–123). Agents will write `calls` edges by hand, and nothing can refuse them. *Replace:* every relation gets `source: [observed: <probe>]` or `[decided|reviewed|user-said]`; observed relations are refused on the write path and materialised from the probe; line 141's `probe:` list becomes the *definition* of observed relations, not a parallel channel.

**D3. `one_sentence: true` text slots are prose in a smaller box.** `contract`, `invariant`, `fails_by`, `shape`, `boundary` (34, 41, 35, 52, 24) are unconstrained strings — annotation properties, unqueryable, uncontradictable, unclosable. `purpose`/`means` may stay annotations. *Replace:* `invariant` → a `gmr-expr` expression over fields (the evaluator exists; `depends` already does this); `fails_by` → set of `(condition, error Type)` edges; `Format.shape` → schema reference; `contract` → `produces/consumes/reads` edges plus an optional gloss. Each slot: structured core + `gloss`.

**D4. `must_change_with` is the symptom of a missing node.** Symmetric, `from: any, to: any`, `reason: required` (118). The reason *is* the concept the two entities share. `Module.boundary` (24) is the same defect: a Policy stored as a text slot, while `layers` migrates as eight Policies. *Replace:* see §3; delete `must_change_with`, derive it.

**D5. Claim has no lifecycle and contradicts itself.** Line 100 says "retired when its basis goes stale"; there is no `standing`, no `retired`, no `about`. Line 103 requires `source: [observed]`; line 183 says an inference *without* an observed source is "a Claim, at most". `caused⁻¹ open Incidents` (138) reads an `open` slot Incident lacks (91–94). *Replace:* Claim = any assertion with `author: agent` and no `reviewed` provenance; `standing ∈ {in, out(since fact_address), retired, promoted_to(Decision)}` derived by the labeller; `about` required; Incident gets `at` and `resolved_by`.

Also: `not_memory` line 180 (discovery → journal) is wrong under PROV: the generating activity is provenance and travels *with* the assertion; the journal is its log, not its home.

## 3. The A/B evidence: what encodes a business relation without a dependency edge

In-repo instance [C]: "a closed anchor is not watched" is implemented at ten sites in seven verb files (`status.rs:44`, `accept.rs:47`, `atlas.rs:57,101`, `mod.rs:118,218`, `revise.rs:130`, `sync.rs:249,535,716`), no shared function, no dependency edge; problems.md §六 shows three verbs already disagree. `cli-read-vs-status.md` reaches two of them only because its `about:` names both.

The node that encodes this in KR is a **Concept** (DDD ubiquitous-language term; in OWL a class in the domain vocabulary, not in the code vocabulary): `Concept:live-anchor`, `definition: an anchor not closed and not terminal`, with `realizes: Function → Concept`. Policies are then `about: Concept`, not about ten Functions. Co-change is a Datalog consequence, not an assertion:

```
must_change_with(X,Y) :- realizes(X,C), realizes(Y,C), X != Y.
must_change_with(X,Y) :- consumes(X,F), produces(Y,F).
```

v1 does not have it. `Format` (49) is the closest — a Concept restricted to data shapes — and `Policy` is the closest for rules, but nothing names *the shared thing itself* when it is neither a shape nor a rule. Missing: `Concept {key: name, slots: definition(structured where possible), gloss}`, `realizes`, and `Policy.about → Concept`. `enter(modify, status.rs#run)` then closes over `realizes → Concept → realizes⁻¹` and returns the other nine sites: the "depth 5" problem in two hops.

## 4. MSIS: when closure under an action signature approximates a sufficient set

Formalisation [O]: task type τ has signature Σ_τ, a set of path rules; `C_Σ(e)` is the least fixpoint from `e`. `C_Σ(e)` is *sufficient* for instance `(τ,e)` iff the correct output is a function of `C_Σ(e)` — any two worlds agreeing on it yield the same answer (Subramanian–Genesereth irrelevance: `KB ∪ X ⊢ Q iff KB ⊢ Q`). This holds under four conditions:

1. **Locality/faithfulness** — every information dependency the task has is an edge type in Σ. A missing relation type (today: Concept) breaks it silently.
2. **Completeness along Σ** — a KG is open-world; absence of `governed_by` means "nobody wrote one", not "none". MSIS needs *negative information*: per-entity, per-relation completeness statements ("`governed_by` of X is complete as of reading R", Darari/Razniewski). Without them "no policy" and "policy not yet written" are indistinguishable — the owner's hallucination trigger. Absent from v1.
3. **Bounded fan-out** — hubs (a Policy governing eight Modules, `Budget` consumed by every call) make closure quadratic. Truncation must be explicit (`+37 more Functions realize this Concept`), never silent — doctor's `never_asked` is the current silent form [M].
4. **Granularity match** — 38 notes about one file [M] means slots were filled at file grain while `modify` acts at function grain; closure at the wrong grain returns everything or nothing.

Noisy slots break minimality linearly; missing provenance makes a stale assertion indistinguishable from a live one.

**Who is right:** both, on different quantifiers. problems.md §零 is right that GMR cannot compute the *instance* MSIS: the task is unknown at `read <key>`. The target direction is right that Σ_τ *defines* a set — but a *type-level* one: `MSIS(τ,e) ⊆ C_{Σ_τ}(e)`, minimal only against the worst instance of τ. Middle ground: `enter(τ, e)` returns `C_{Σ_τ}(e)` with completeness markers and truncation counts, claims sufficiency-for-τ, never minimality; the agent narrows. Σ_τ is then *validated empirically*: the `usage` table (schema V14→V15) and `saw` already record what was delivered vs cited; a relation in Σ_modify that no `modify` task ever cites is pruned. That is the owner's convergence metric, measurable today.

One constraint is not in the ontology at all [M, problems.md §八]: the same information is a redirect before the agent's plan forms and a footnote after. `enter` must run at task start, by hook, or MSIS is moot.

## 5. Write protocol: the atomic unit

Not a triple (no provenance), not a note (a bundle; migration-trial gap 2). The unit is a **nanopublication**: one assertion graph (one slot-fill or one relation instance, or a small set about one entity), one provenance graph, one publication-info graph. Serialise line-oriented, not JSON: the envelope is the 5× problem [M].

`runtime-carry-linked.md` after migration (real addresses; `fa:` = fact_address prefix):

```
np t4711/3 at 2026-09-08T10:41Z by agent:t4711 src reviewed:debcd93
  cites fa:ast-map/memory.rs#carry_linked@<hash> fa:ast-map/read.rs#ground@<hash>
Function gmr-runtime/memory.rs#carry_linked
  contract  = follows links one hop from delivered memories; opt-in; carried records grounded=false; budget narrowed from the caller's total
  reads     -> Field gmr-runtime/read.rs#Instructions.carry
  governed_by -> P:content-budget
  decided_by  -> D:carry-one-hop
  verified_by -> Test gmr-runtime/tests/grounding.rs#carrying_linked_records_is_asked_for_and_they_come_back_marked
  verified_by -> Test gmr-runtime/tests/grounding.rs#an_unanchored_record_is_carried_along_but_marked
  realizes    -> Concept:delivery-relevance
  complete    governed_by decided_by verified_by
D:carry-one-hop by human:zongming src decided
  claim  = relevance beyond one hop is the domain's judgment, not the substrate's
  rationale = a store read per record on every read path is not paid for unasked; mixing "mentions" into "about" dilutes attention
  review = does carry_linked recurse, mark a carried record grounded=true, or run without carry asked?
  contrasts -> Function gmr-runtime/link.rs#reaching   # propagation, not delivery
```

≈900 bytes against 3,029 bytes of note text inside an 18,580-byte envelope [M]. What the note said that is not above is history (journal) or an edge (`contrasts`). The `complete` line is the negative information §4 requires; `cites` is what the labeller watches; `src reviewed:<commit>` is what makes it a premise rather than a Claim.
