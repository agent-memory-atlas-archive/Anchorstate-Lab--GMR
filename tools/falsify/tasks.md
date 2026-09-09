# Ten tasks on gmr-core, pre-registered

Written before any run, from design.md §5.2 and §4.4. Each task is real work Phase 1
needs. The agent gets exactly the text under its heading, nothing else about the task.
Acceptance for every task: `cargo test -p gmr-core` passes; the behaviours listed have
tests; no comments in code; nothing is committed.

## Task 1

In `crates/gmr-core/src/anchor.rs`, give `Transitions` a method `content_hash(&self) -> ContentHash`
that hashes the ordered list of its rules, each rule as its guard hash and its new-state hash,
through the crate's canonical content hash. Tests: two tables with the same rules in the same
order hash the same; the same rules in another order hash differently; the empty table has a hash.

## Task 2

In `crates/gmr-core/src/journal.rs`, `Entry::Open` gains a field `rules: Option<ContentHash>`
that older entries on disk, written without it, still deserialise (it reads as `None`).
`AnchorState` gains `rules: ContentHash`: the entry's value when present, otherwise the hash
of the anchor's transitions. Tests: an Open entry serialised without the field still folds;
the folded `rules` equals `anchor.transitions.content_hash()`; the existing property that
resuming from every cut equals folding from zero still holds with the new field.

## Task 3

In `crates/gmr-core/src/journal.rs`, `Change::Retransition` gains `rules: Option<ContentHash>`
with the same backward-compatible deserialisation as Task 2, and applying it updates
`AnchorState.rules` (the given hash when present, otherwise the hash of the new transitions).
Tests: old JSON still parses; after a Retransition the folded `rules` is the new table's hash.

## Task 4

In `crates/gmr-core/src/journal.rs`, the `context` field of `Entry::Revise` and of `Entry::Close`
becomes `Option<ContentHash>`, absent by default, so a revise or close no longer has to seal a
state snapshot. Tests: entries written with and without `context` both deserialise; folding is
unaffected by its presence.

## Task 5

In `crates/gmr-core/src/journal.rs`, add tests that pin what `Entry::Still` and `Entry::Attempt`
do to the fold: a Still after an Attempt clears the faltering count and advances `last_sighting`
but does not move `entered_at`; an Attempt after a Still counts as the first attempt of a new
run; a Still never changes `state`. Fix the fold only if a test shows it wrong.

## Task 6

In `crates/gmr-core/src/addr.rs`, add `hash_at(value: &Value, path: &str) -> Option<ContentHash>`:
`path` is dotted, a segment that parses as an unsigned integer indexes an array, the empty path
names the whole value, and an absent path yields `None`. Tests: a nested object field; an array
element; an absent field; `hash_at(v, "")` equals `content_hash_of(v)`; a numeric key on an
object is a field name, not an index.

## Task 7

In `crates/gmr-core/src/anchor.rs`, give `State` two methods: `at(&self, path: &str) -> Option<&Value>`
with the same path rules as Task 6, and `hashes_at(&self, paths: &[&str]) -> Vec<(String, Option<ContentHash>)>`
returning one entry per requested path in the order given. Tests: present and absent paths;
`position` and `status` reachable by path like any other field.

## Task 8

In `crates/gmr-core/src/addr.rs`, add `paths_differing(before: &Value, after: &Value) -> Vec<Differing>`
where `Differing { path: String, how: How }` and `How` is `Changed`, `Added` or `Removed`,
walking objects by key and arrays by index, reporting leaf paths only, sorted by path. Tests:
a changed leaf; a renamed key reports one Removed and one Added; an array with an element
changed reports that index; an element inserted at the front reports every later index;
identical values report nothing.

## Task 9

In `crates/gmr-core/src/journal.rs`, give `AnchorState` a method
`stale_paths(&self, recorded: &[(String, ContentHash)]) -> Vec<(String, Stale)>` where `Stale`
is an enum with `Value` and `Absent`: for each recorded path, `Absent` when the current state
has no value there, `Value` when it has one whose hash differs, nothing when it matches.
Tests: unchanged paths are not reported; a changed path is `Value`; a path that vanished is
`Absent`; the result preserves the order of `recorded`.

## Task 10

Extend Task 9: `stale_paths` takes a further argument `recorded_derivation: Option<&ProbeVersion>`.
When it is given and differs from the derivation version of the anchor's latest observation,
a path whose value differs is reported as `Stale::Instrument` instead of `Stale::Value`; a path
whose value matches is still not reported; when the versions agree, or none is given, nothing
changes from Task 9. Tests cover all four combinations of version agreement and value agreement.
