#!/usr/bin/env bash
# Findability acceptance: a fresh agent, holding only the gmr binary and its
# SKILL.md, must assemble the task's minimal sufficient information set by
# walking the anchor-memory network. Runs claude -p, so it costs API money and
# stays out of CI; run it by hand and paste the reports into the PR.
#
#   GMR_BIN=/path/to/gmr SCENARIOS="s1 s2 s3 s4" tools/msis/run.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
GMR_BIN="${GMR_BIN:-$REPO/target/release/gmr}"
CLAUDE_BIN="${CLAUDE_BIN:-claude}"
SCENARIOS="${SCENARIOS:-s1 s2 s3 s4}"
OUT="${OUT:-$REPO/.anchor/output/msis}"
mkdir -p "$OUT"
export GMR_BIN

json_contract='End your reply with one JSON object:
{"answer": "...", "governing_constraints": ["..."], "affected_conclusions": ["..."], "relied_on": ["<provider:id or said:id addresses>"], "verbs_used": ["..."]}'

prompt_common="This repository is managed by gmr; read .claude/skills/gmr/SKILL.md first and use the gmr verbs to gather exactly the information you need. The gmr binary is at $GMR_BIN. Do not modify any file or accept anything. Cite records by the addresses gmr prints."

run_llm() {
  local scenario="$1" mutation="$2" task="$3"
  local dir; dir="$(mktemp -d)/repo"
  python3 "$HERE/fixture.py" "$dir" "$mutation" >"$OUT/$scenario.fixture.json"
  ( cd "$dir" && "$CLAUDE_BIN" -p "$prompt_common

Task: $task

$json_contract" --allowedTools Bash Read Grep Glob ) >"$OUT/$scenario.transcript.txt" 2>"$OUT/$scenario.stderr.txt" || true
  ( cd "$HERE" && python3 grade.py "$scenario" "$OUT/$scenario.transcript.txt" ) | tee "$OUT/$scenario.report.json"
}

s4_mechanical() {
  local dir; dir="$(mktemp -d)/repo"
  python3 "$HERE/fixture.py" "$dir" s4 >"$OUT/s4.fixture.json"
  set +e
  ( cd "$dir" && "$GMR_BIN" check --json ) >"$OUT/s4.check.json" 2>&1
  local code=$?
  set -e
  local unclaimed
  unclaimed="$(python3 -c "import json,sys; d=json.load(open('$OUT/s4.check.json')); print(len(d.get('unclaimed',[])))")"
  echo "{\"scenario\": \"s4\", \"check_exit\": $code, \"unclaimed\": $unclaimed}" | tee "$OUT/s4.report.json"
  if [ "$code" -ne 1 ] || [ "$unclaimed" -lt 1 ]; then
    echo "s4 FAILED: an unbound anchor moved, another verb consumed the edge, and check stayed green" >&2
    return 1
  fi
}

for s in $SCENARIOS; do
  echo "== $s =="
  case "$s" in
    s1) run_llm s1 "" "A change request asks to raise the discount in src/rates.rs from 250 to 300 basis points. What governs this change, is it allowed, which recorded conclusions would it invalidate, and what would checkout::total(10_000) return afterwards?" ;;
    s2) run_llm s2 "" "You are about to refactor src/checkout.rs. Before touching total(), assemble everything this repository knows that constrains it: the notes it rests on, and any recorded conclusion that would lose its ground. Then state what total(10_000) would return if the discount rose to 300 bps and whether that rise is allowed." ;;
    s3) run_llm s3 s3 "Someone edited src/rates.rs recently. Is the recorded conclusion cart-total-baseline still reliable, and what does gmr report about the notes on the discount? Answer with reliable true or false in the answer field." ;;
    s4) s4_mechanical ;;
  esac
done
echo "reports in $OUT"
