# falsify — Phase 0 of design.md, before any runtime code

Ten real gmr-core tasks (tasks.md), two arms, one fresh `claude -p` per task and arm.
The net arm has the Phase 0 net (tools/net: walk, write, check over line-grammar
containers seeded from the gmr-core notes); the ctl arm has today's SKILL.md and gmr.
The verdict is pre-registered in criterion.md and computed by grade.py.

```
cargo build --release -p gmr-cli
tools/falsify/all.sh                     # ten tasks, both arms, then the verdict
ARM=net TASK=3 tools/falsify/run.sh      # one cell
python3 tools/falsify/grade.py .anchor/output/falsify/results.jsonl
```

Working copies live under `.anchor/output/falsify/work-<arm>/repo`; the net under
`work-net/net`. Both accumulate across tasks. Delete the output directory to start over.
Costs API money; never in CI.
