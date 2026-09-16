---
about: crates/gmr-core/src/addr.rs#canonical_number_string
watch: [sig, logic]
---

# How a number is written is part of the hash

One value has many spellings: `1.50` `1.5` `1.5e0` `-0`. Canonicalization has to
collapse them into one, or two JSON documents with the same content hash to two
different addresses and `ContentHash` stops being the address of the content.

The reverse failure is the sharper one, and it is what shapes the code: two
*different* values must never collapse into one spelling. `1.5e20` and `1.5e200`
did, because the strip-trailing-zeros rule ran over the whole formatted string
whenever it held a decimal point, and ate the zeros inside the exponent along
with the mantissa's.

So the formatted value is **split at the exponent marker first**. Everything
after it is carried through untouched; only the mantissa is trimmed. The rules
that survive the split: integers go through `to_string()` and never touch the
float path; trailing `0`s after a decimal point and a lone trailing `.` are
dropped from the mantissa; `-0` folds to `0` once, after the trim, rather than by
matching two spellings before it.

Floats are formatted with ryu, which emits a lowercase `e` and no `+`, so no
case or sign normalization of the exponent is needed — and none is done. The
split still looks for `E` because the thing it must not do is trim across a
marker it failed to recognise.

**If ryu or serde_json quietly changes its format, every historical hash in this
repository stops matching** — the test
`canonical_form_is_pinned_against_library_drift` pins the bytes and the hash of
one fixed value so that kind of drift explodes when the dependency is upgraded,
not on the day someone compares against an old log.

That `unreachable!` at the end is not laziness: without `arbitrary_precision`,
`serde_json::Number` has only PosInt / NegInt / Float, and `as_f64()` is total on
all three. Turn that feature on and this line really is reachable — so it is an
invariant **that depends on a feature**.

## When this changes, ask

Any format rule changed → every historical `ContentHash` is void and logs cannot
be compared backwards. That is not an "improvement", it is a breaking change, and
it has to be treated like swapping a probe version.

Any trimming that reaches past the exponent marker, or any rule that reads the
formatted value as one flat token → ask which two distinct values now share a
spelling. That is the question the split exists to answer, and it is not
answered by testing round numbers.

`arbitrary_precision` gets turned on indirectly by some dependency → the
`unreachable!` becomes a panic. Ask why it was turned on before deciding whether
to turn it off or to give this a real branch.
