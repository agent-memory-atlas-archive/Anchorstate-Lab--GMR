---
about:
  - console/cli/src/delivery.rs#delivers
  - console/cli/src/delivery.rs#a_note_that_says_nothing_takes_its_shapes_default
  - console/cli/src/delivery.rs#an_anchor_with_no_shape_and_no_watch_refuses_to_guess
watch: [sig, logic]
---

# Delivery asks "is anything still unhandled", not "did it move this time"

`check` used to recognise only `Observed::Transitioned`. A second observation where
the state had not moved was `Still` — nothing reported, exit code 0. **The
accumulated bits were fed to `status` and not to delivery.** Decision 1's "once set,
stays set until the person re-confirms" was honoured in the display layer and not in
the delivery layer.

The consequence was measured: this repository's `doctrine::red-cards` was broken (the
section it watched did not exist), `doctor` printed `section-gonex1`, and `check`
exited 0 — **CI was green**. After a signature change, running check twice gave
"nothing moved" the second time while `status` still had `v.sig` raised.

There are three paths now, **divided by what is declared, not by guessing from what
the state looks like**:

| | criterion | who decides |
|---|---|---|
| a `watch:` on the note or the anchor | the compiled expression over the state | its author |
| a shape and no `watch:` | is any subscribed bit set | bits accumulate; `accept` clears them |
| hand-written rules and no `watch:` (`of()` returns `None`) | refuse to answer | nobody declared when the memory should return |

The refusal in the third row replaced an edge fallback that once lived there.
Delivering on the transition edge announced the obligation once and lost it;
staying quiet lost the memory. Neither is an answer this layer may invent, so the
missing subscription is reported against the declaration (`watch-missing`) and a
delivery question that still arrives is a snag, not a verdict.

An expression that evaluates to `Absent` is a snag for the same reason: an absent
answer is the expression failing to decide, not the memory deciding to stay quiet,
and swallowing it as "no" was the one unevaluable outcome this layer converted
into a verdict. The evaluator short-circuits `and`/`or`, so the canonical
`exists(state.v.x) and state.v.x` guard still reads absence as a decided no.

There was once a third — table shapes leaned on a hand-written `settled` allowlist.
That was a product of the dual track: two answers to one question (does this state
still need a human), and the allowlist one had no subscriptions. Once every built-in
shape was vectorized it disappeared: settled means **every bit down**, derived rather
than listed.

**It asks the declaration, not the data.** `delivers` takes an `Option<&Shape>` and no
longer infers the kind of shape from whether the `state` has a `v` — that is
structural typing, and under it a hand-written-rules anchor and a table shape are
indistinguishable.

## A subscription is keyed by a `Ref`, not by a bare id

`delivers` takes the note's full address — provider and external id —
because a subscription belongs to one record in one store. Keyed by the
bare id, a note in a second store silently inherits the narrowing of a note
it merely shares a name with.

That failure is worth spelling out because of how it looks: the memory
simply stops being handed back, which is indistinguishable from the axis
not having moved. Nothing prints, nothing turns red, and the anchor reads
as settled. It is the quietest way this repository can fail, which is why
the case has its own test rather than resting on the type change alone.

## When this changes, ask

Adding an axis → ask: **after a person has seen this bit lit, is there anything left
for them to do?** If what they do matches an existing axis → merge them (criterion in
[[shapes-Dim]]). If there is nothing at all for them to do → it should not be an axis,
because settled means every bit down, and a bit that never needs a human keeps the
anchor unsettled forever.

There used to be a `settled` allowlist here, enumerating which statuses counted as
settled. It and the bit vector were two answers to one question, and the allowlist one
had no subscriptions — add a status to the vocabulary and forget the allowlist and the
anchor is handed back forever; forget to remove one and the anchor goes silent
forever. **One of two answers always gets forgotten, so keep only the derived one.**

The refusal gets softened (say, "if nothing is declared, call it settled") →
hand-written-rules anchors go silent forever. The reverse, "call it unsettled" →
they exit 1 forever. Both are wrong, which is why the undeclared case answers with
neither. And "do not recognise" has a second meaning — the criteria drifted; see
[[check-drift]].
