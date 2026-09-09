#!/usr/bin/env bash
# All ten tasks, both arms, net first on every task. Then the verdict.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
OUT="${OUT:-$REPO/.anchor/output/falsify}"
FROM="${FROM:-1}"
TO="${TO:-10}"
for T in $(seq "$FROM" "$TO"); do
  for ARM in net ctl; do
    ARM="$ARM" TASK="$T" OUT="$OUT" "$HERE/run.sh"
  done
done
python3 "$HERE/grade.py" "$OUT/results.jsonl"
