# Protocol review: wire and write, token budgets

**(M)** measured · **(C)** code · **(O)** opinion.

## 1. Read protocol

**18,564 B accounted** (`read.json`) **(M)**:

| bytes | field | needed by |
|---|---|---|
| 9,265 | `anchor.transitions`: 11 rules, 22 sha256 (1,650 B hex); generated from `shape: contract` **(C)** | nobody |
| 1,094 | `baseline` + `now`, identical when settled; the 329-B `sig` appears 3× | auditor |
| 1,283 | `facts` (restates `now`) + `derivation` | nobody; agent uses `found`, `line`, `fact_address` |
| 680 | per-memory envelope: versions, seqs, `sources`, `warrant`, `grounded` (inverted), `links: []` | auditor; the agent bit is `moved sig,body @62437` |
| 2,837 | note text | agent, but ~50% fills no slot (migration-trial) |
| 190 | key, status, timestamps, counters | agent: key+status |

Agent-needed: key, status, one stale bit, address, the note's slot-worthy half ≈ 1.6 KB: **envelope is 11:1, not 5:1.** `--lean` is 15,639 B with zero memory text **(M)**: the declaration must never leave the store on a read.

**Target: line-oriented text, fixed grammar.** JSON spends ≥4 tokens per field on punctuation and repeats keys; YAML is ambiguous as a *write* format; Markdown has no grammar — the 196 notes are what that yields. A line grammar parses with `split`, and **read and write share it** **(O)**.

```
@Type key [=addr] ~s      node; =addr is the reading, 8 hex
slot: text ~s             leading ! = the source reading moved
>rel Type:key ~s  <rel …  edge out / in; indented lines = far node's slots
~s who how address [!stale axes @seq]   provenance, once per source
```
Provenance: a 2-byte `~x` per line, resolved once in the footer. Stale: `!` on the line and its source. Field order follows the action signature. Elided: rule tables, hashes past 8 hex, `baseline`/`now`, `facts`, `derivation`, sig strings, timestamps, `grounded`, empty lists.

Worked example, 1,094 B, ~290 tokens; the stale mark is today's real `holding=moved @62437` **(M)**:
```
@Function crates/gmr-runtime/src/memory.rs#carry_linked =fcfcce7f ~a
purpose: carries records linked from an already-delivered memory into the answer, one hop, when asked. ~a
!contract: gated by Instructions.carry; narrows a slice of the caller's total Budget per record and mints no total of its own. ~a
fails_by: a store call that outruns its slice is reported unreachable, never dropped silently. ~a
>reads Field:crates/gmr-runtime/src/read.rs#Instructions.carry ~a
>governed_by Policy:content-budget ~b
 claim: one operation has one content total; every walk narrows a slice of it.
>decided_by Decision:carry-one-hop ~b
 claim: relevance beyond one hop is the domain's judgment, not the substrate's.
 ask: does the new code recurse past one hop, or walk without carry being asked?
>verified_by Test:crates/gmr-runtime/tests/grounding.rs#carrying_linked_records_is_asked_for_and_they_come_back_marked ~a
<calls Function:crates/gmr-runtime/src/read.rs#ground ~p
~a agent task-0912 observed fcfcce7f@61850 !stale sig,body @62437
~b human zongming decided 2026-08-22
~p probe callers,tests now
```
Envelope (footer, refs, sigils) = 154 B = **14%**. Budget: ≤ 20% envelope, cap 4 KB with a closing `…+N Type` line. 18,564 → 1,094 is 17×; a `%prefix` header for relative keys reaches ~850 B **(O)**.

## 2. Write protocol

`gmr write < doc` (`--task <id>`, `--dry-run`). Same grammar; many blocks per document; **atomic**.

Validated, in order: (1) `@Type` in the ontology, key matches the type's form; (2) fact-keyed entities routed to their probe **now** — a miss is `unresolved`, a hit yields the address; Decision/Policy created if absent; (3) every slot declared, `required` present on creation; (4) source legal for the slot (Decision `claim`: only `decided`, `human <name>`); `observed` must cite an address this store issued — today's `Shown::Unseen`, promoted to refusal; (5) `one_sentence`; (6) relation kind exists, from/to types match, `must_change_with` needs `reason:`, `supersedes` an existing `same_kind` target; (7) §5.

Reply, 137 B:
```
+ Decision:carry-one-hop
+ Decision:carry-is-opt-in
+ Decision:carry-shares-budget
+ Function:…#carry_linked.contract @62501
+ 4 edges
```
Refusal: `! line 7 one_sentence`, exit 2; warnings land, exit 1.

The call for `runtime-carry-linked.md`; migration-trial compresses it to one D plus a contract slot, but the note holds three judgments with three review questions **(O)**:
```
@Decision carry-one-hop ~h
claim: relevance beyond one hop is the domain's judgment, not the substrate's.
rationale: what should I hand this reader gets longer and vaguer with every hop, and the substrate has no basis for relevance.
ask: does the new code recurse past one hop?
>decided_by⁻¹ Function:crates/gmr-runtime/src/memory.rs#carry_linked
@Decision carry-is-opt-in ~h
claim: the linked walk runs only when Instructions.carry asks; silence means not at all.
rationale: a store read per record on every read path is a cost no caller should pay unasked, and mixing mentions into aboutness dilutes attention.
ask: does the walk run without carry being asked for?
>decided_by⁻¹ Function:crates/gmr-runtime/src/memory.rs#carry_linked
>decided_by⁻¹ Field:crates/gmr-runtime/src/read.rs#Instructions.carry
@Decision carry-shares-budget ~h
claim: carried records narrow a slice of the caller's total budget and mint no second total.
rationale: a second total lets one read spend twice what its caller allowed, on the half of the walk nobody asked for record by record.
ask: does carry_linked take a budget that is not a slice of the caller's total?
>decided_by⁻¹ Function:crates/gmr-runtime/src/memory.rs#carry_linked
>governed_by⁻¹ Policy:content-budget
@Function crates/gmr-runtime/src/memory.rs#carry_linked ~a
contract: gated by Instructions.carry; narrows a slice of the caller's total Budget per record and mints no total of its own. ~a
>reads Field:crates/gmr-runtime/src/read.rs#Instructions.carry ~a
~h human zongming decided 2026-08-22
~a agent task-0912 observed fcfcce7f
```
1,597 B in, 137 B out, against a 2,853 B note plus `sync`. Not written: the note's "marks every one `grounded: false`", false since written (problems.md §三); `grounded` is not an identifier in `carry_linked`'s body, so check 6 flags it.

## 3. Carrier

Yes, on two conditions **(O)**. SKILL.md is 28,258 B **(M)**: ~60% describes 34 verbs, providers, mem0, doctor's JSON; ~25% restates CLAUDE.md §1; the binding contract is under 3 KB. The ontology (8,109 B) is read by the runtime, so it cannot drift. Condition one: action signatures must be executable — `calls⁻¹ | reads⁻¹`, "governed_by of every contains⁻¹ ancestor" are comments the parser skips; a six-operator path language (`rel`, `⁻¹`, `|`, `->`, `*`, filter) makes `enter` the interpreter. Condition two: nobody loads it up front — `enter` names its slots, refusals name the rest, `read ontology.Type` answers the remainder. Fixed cost: ~42 KB (~10k tokens) → 1,535 B (~380 tokens). CLAUDE.md §2–§3 become one `Policy` node each.

Entry card, 1,535 B:
```
---
name: gmr
description: Trigger when `.anchor/` exists. Before changing code, `gmr enter`; when done, `gmr write` what fills a slot.
---
# gmr in five verbs

    gmr locate <words|path#name>    -> ≤5 lines: Type key
    gmr enter <action> <key>        -> the subgraph an action needs (modify|extend|explain|verify|debug)
    gmr read <key>|ontology[.Type]  -> one node; or the slots and relations a Type has
    gmr write < doc                 -> land slots, edges, Decisions; refuses noise
    gmr check [key]                 -> only what went stale; silent when nothing did

Wire grammar, same for read and write:

    @Type key ~s            a node; ~s names a source defined at the end
    slot: one sentence. ~s  `!` in front: its reading moved since it was written
    >rel Type:key ~s        edge out (`<rel` edge in); indented lines = far node's slots
    ~s who how address      `agent <task> observed <addr>` | `human <name> decided <date>`

Rules the runtime enforces; a refusal names the line and the code:
- only what fills a declared slot; `gmr read ontology.<Type>` lists them
- `observed` needs an address that enter/read handed you this task
- slot text: one sentence; no dates, units, "used to/once/no longer", [[links]], quoted code. A cross-reference is an edge line.
- Decision/Policy carry claim, rationale, ask; a person decides them, never you
- your own inference is a Claim with `>rests_on <addr>`; check retires it when the address moves

Exit: 0 done · 1 stale or warned · 2 refused, nothing written
```

## 4. Verb surface

Today: 16 shown, 18 hidden **(C)**.

| verb | input | output | exit | absorbs |
|---|---|---|---|---|
| `locate` | words, `path#name`, `path:line`; `--type`, `-k` | ≤ 5 lines, ≤ 400 B | 0 hit · 1 none · 2 error | `memories`, `links`, `atlas`, `status` listing |
| `enter` | action, key, `--budget` | ≤ 4 KB, envelope ≤ 20% | 0 · 1 any `!` · 2 unresolved · 3 no such action | `read --carry`, `status <key>`, `check <key>`, `sample`, `observe`, `ground --reach` |
| `read` | key or `ontology[.Type]` | ≤ 1 KB, no traversal | 0 · 2 | `read`, `since`, `health` |
| `write` | stdin doc | ≤ 200 B, ≤ 80 B/refusal | 0 · 1 warned · 2 refused | `anchor -m|--record`, `said`, `bind`, `attest`, `reaffirm`, `cobound`, `link`, `condense`, `accept --why` (= rewrite the slot with a fresh `observed`), `close` (= `retire`), revise family |
| `check` | nothing, key, `--since`, `--task` | ~100 B per stale item, **zero when quiet** | 0 quiet · 1 stale · 2 probe failed | `check`, `observe`, `pass`, `ground`, `doctor`, `sync` |

No place: **`said`/`ground`** — a Claim with `>rests_on addr`; address staleness replaces `--depends`; losing the narrow invariant is deliberate, since `watch:` was the same idea and hid 15 of 17 movements (problems.md §三). **`condense`** — a human `write` with `supersedes`. **`atlas`** — a renderer over `read`. **`adopt`** — task writes fill the net. **`accept`/`close --why`, revise family, `open`, `rebase`** — drive a rule table `shape` generates; `--why` is the `rationale` slot. **`sync`** — one declaration channel, nothing to reconcile (§二). `init`, `probes`, `export`, `import`, `publish`: operator bootstrap, hidden.

## 5. Noise rejections at write time

| # | check | catches | false-positive risk |
|---|---|---|---|
| 1 | one terminator outside backticks, ≤ 240 chars | bundles, narrative | low |
| 2 | history lexicon outside backticks (*used to, once, previously, no longer, briefly, originally, has not changed*) | "used to check the token" | medium; refuse on `claim`/entity slots, warn on `rationale` |
| 3 | ISO dates, `\d+ ?(ms|KB|MB|%|of \d+)` | "76.3KB", "580/600" | medium; exempt `Config.controls`, `Field.unit` |
| 4 | `[[…]]`, `](`, "see ", "as X says" | every prose cross-reference | ~zero |
| 5 | ≥ 40-char substring of the entity's own source; > 2 backtick spans | field-by-field explanations | low |
| 6 | backticked identifiers must be in the entity's reading or resolve to a key | `grounded: false` on `carry_linked` | medium-high; **warning, exit 1** |
| 7 | *we, I, probably, seems* in `claim` | diary notes | low; off for `Claim.text` |
| 8 | *because, so that, since* in `claim` | rationale leaking into claim | low |
| 9 | stem-Jaccard ≥ 0.8 with an existing slot, no `supersedes` | 38 notes on `read.rs` restating each other | medium; refusal names the id |

Checks 2–5 cover the trial's four drop classes; on `runtime-carry-linked.md` they refuse paragraphs 3–4 and the `[[…]]` lines — the half the trial dropped **(C)**. Not caught: a false one-sentence contract with a legal source — entailment; check 6 only warns.

