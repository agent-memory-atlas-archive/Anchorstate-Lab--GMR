# msis — findability acceptance

A fresh agent (`claude -p`), holding only the gmr binary and its SKILL.md, must
assemble a task's minimal sufficient information set by walking the
anchor-memory network of a synthetic repository. The infrastructure bar is
**found, even if slowly** — retrieval speed and precision are reported as
metrics, never gated.

The ground truth (see `fixture.py`): two notes joined by a `rests-on` link
across two anchors, one journal-only conclusion citing both, three distractor
notes. The vendor name and contract cap live only in one note's body; the
conclusion lives only in the journal, so no file grep can complete the set.

| scenario | entry | proves |
|---|---|---|
| s1 | the rates coordinate | incoming link + cobound walk reaches the other anchor's note and the conclusion |
| s2 | the checkout coordinate | outgoing link walk, same set from the other end |
| s3 | after an unannounced edit | the drifted ground is reported, not silently relied on |
| s4 | mechanical, no LLM | an unbound anchor stays due after another verb consumed its transition edge |

Run by hand (each LLM scenario is one `claude -p` call and costs API money;
this never runs in CI):

```
cargo build --release -p gmr-cli
tools/msis/run.sh                      # all four
SCENARIOS="s4" tools/msis/run.sh       # mechanical only
GMR_BIN=/path/to/other/gmr tools/msis/run.sh   # baseline another build
```

Reports land in `.anchor/output/msis/`. Gates: sufficiency (the answer contains
what only the full set yields) and findability (`relied_on` covers the truth
addresses). The agent is granted unrestricted Bash inside a throwaway fixture
directory — run it on machines where that is acceptable.
