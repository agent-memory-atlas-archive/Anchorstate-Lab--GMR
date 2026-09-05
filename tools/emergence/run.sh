#!/usr/bin/env bash
# Emergence experiment: does the memory layer grow an A-B edge the code graph
# does not have, without anyone being told to connect them — and can a later
# agent entering at A reach B through it? Three claude -p runs; by hand only.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
GMR_BIN="${GMR_BIN:-$REPO/target/release/gmr}"
CLAUDE_BIN="${CLAUDE_BIN:-claude}"
MODEL_ARGS=()
[ -n "${MODEL:-}" ] && MODEL_ARGS=(--model "$MODEL")
OUT="${OUT:-$REPO/.anchor/output/emergence}"
STAGE="${STAGE:?set STAGE=s1|s2|control}"
DIR="${DIR:?set DIR=<fixture dir>}"
mkdir -p "$OUT"
export GMR_BIN

json_tail_s1='End with one JSON object: {"root_cause": "...", "fix": "...", "gmr_left_behind": ["..."], "relied_on": ["..."]}'
json_tail_s2='End with one JSON object: {"affected": ["..."], "constraints": ["..."], "relied_on": ["<addresses gmr printed>"], "verbs_used": ["..."]}'

common="This repository is managed by gmr; read .claude/skills/gmr/SKILL.md first. The gmr binary is at $GMR_BIN. Cite records by the addresses gmr prints. Do not commit or push."

s1_task="Finance reports that the reporter's monthly totals are roughly 100x too small for recent entries; older months reconcile fine. Sample feed data is under data/feed/. Diagnose the root cause and apply the minimal code fix. Follow the discipline the skill describes while you work, and leave behind whatever a maintainer following that discipline should leave behind about what you learned. $json_tail_s1"

s2_task="Change request for crates/exporter: every ledger line written by write_entry should gain a fourth field carrying an ISO currency code. Before any implementation, answer: what constrains this change, and who or what would be affected if the line shape changes? Do not modify any file. $json_tail_s2"

case "$STAGE" in
  s1)
    python3 "$HERE/fixture.py" "$DIR" >"$OUT/fixture.json"
    ( cd "$DIR" && "$CLAUDE_BIN" ${MODEL_ARGS[@]+"${MODEL_ARGS[@]}"} -p "$common

Task: $s1_task" --allowedTools Bash Read Grep Glob Edit Write ) \
      >"$OUT/s1.transcript.txt" 2>"$OUT/s1.stderr.txt" || true
    ( cd "$HERE" && python3 detect.py "$DIR" ) | tee "$OUT/s1.detect.json"
    ;;
  s2)
    ( cd "$DIR" && "$CLAUDE_BIN" ${MODEL_ARGS[@]+"${MODEL_ARGS[@]}"} -p "$common

Task: $s2_task" --allowedTools Bash Read Grep Glob ) \
      >"$OUT/s2.transcript.txt" 2>"$OUT/s2.stderr.txt" || true
    ;;
  control)
    python3 "$HERE/fixture.py" "$DIR" >"$OUT/control.fixture.json"
    ( cd "$DIR" && "$CLAUDE_BIN" ${MODEL_ARGS[@]+"${MODEL_ARGS[@]}"} -p "$common

Task: $s2_task" --allowedTools Bash Read Grep Glob ) \
      >"$OUT/control.transcript.txt" 2>"$OUT/control.stderr.txt" || true
    ;;
esac
echo "stage $STAGE done; outputs in $OUT"
