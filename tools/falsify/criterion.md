# Pre-registered criterion, written before any run

design.md §14. Two arms run the same ten tasks (tasks.md) in the same order on the same
code, each task a fresh `claude -p` session, model `sonnet` unless MODEL says otherwise.

## Arms

- **net**: the agent has the Phase 0 net (seeded from the gmr-core notes by
  tools/net/render.py, growing with every write) and three commands: walk, write, check.
  The gmr skill file is removed from its working copy; it still has the gmr binary for
  readings. Its entry card is entry-card.md.
- **ctl**: the agent has today's channel: `.claude/skills/gmr/SKILL.md` and the gmr binary.

Both arms have every tool (Read, Grep, Glob, Edit, Write, Bash) and the whole repository.
Code accumulates within an arm when a task's tests pass and is reset to the last passing
state when they fail. The net accumulates across tasks in the net arm; the ctl arm's
writes go wherever today's verbs put them.

## The four numbers, per task and arm

1. **outside**: bytes of tool results that entered the context before the first edit and
   did not come through the memory channel. The channel is walk/write/check output for
   the net arm; gmr output and SKILL.md for the ctl arm. Measured by meter.py over the
   stream-json transcript.
2. **tests**: whether `cargo test -p gmr-core` passes in the working copy after the run.
3. **valid**: net arm only, the share of write attempts in this task that landed (exit 0 or 1)
   among all attempts, from NET/writes.jsonl. The ctl arm's write-verb invocations are counted
   but not judged.
4. **stale**: net arm only, the number of records whose rests_on address moved, from
   check.py after the task's readings were refreshed. Observed, not judged.

## Verdict

The design is falsified if, in the net arm, either holds:

- the mean of `outside` over tasks 6–10 is not below 70% of the mean over tasks 1–5, or
- fewer than half of all write attempts across the ten tasks landed.

Otherwise it survives this experiment. `tests` and `stale` are reported alongside and do
not enter the verdict. Nothing here is tuned after seeing transcripts.
