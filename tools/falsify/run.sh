#!/usr/bin/env bash
# One task, one arm, of the Phase 0 falsifier (design.md §14, criterion.md).
#
#   ARM=net|ctl TASK=1..10 [MODEL=sonnet] [OUT=.anchor/output/falsify] tools/falsify/run.sh
#
# Runs claude -p, so it costs API money and stays out of CI. The net arm must run task 1
# before the ctl arm runs anything: it seeds the net and the key list both arms refresh.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
ARM="${ARM:?net|ctl}"
TASK="${TASK:?1..10}"
OUT="${OUT:-$REPO/.anchor/output/falsify}"
MODEL="${MODEL:-sonnet}"
GMR_BIN="${GMR_BIN:-$REPO/target/release/gmr}"
CLAUDE_BIN="${CLAUDE_BIN:-claude}"
TIMEOUT="${TIMEOUT:-2400}"
WORK="$OUT/work-$ARM/repo"
NET="$OUT/work-net/net"
RUN="task$TASK-$ARM"
GIT_ID=(-c user.email=falsify@example.invalid -c user.name=falsify)
mkdir -p "$OUT/work-net"
export GMR_BIN

if [ ! -d "$WORK" ]; then
  git clone -q "$REPO" "$WORK"
  git -C "$WORK" checkout -q "$(git -C "$REPO" rev-parse HEAD)"
  mkdir -p "$WORK/.anchor/state"
  cp "$REPO/.anchor/state/memory.db" "$WORK/.anchor/state/"
  for f in extract-cache.json survey-index.sqlite; do
    [ -f "$REPO/.anchor/state/$f" ] && cp "$REPO/.anchor/state/$f" "$WORK/.anchor/state/"
  done
  if [ "$ARM" = net ]; then rm -rf "$WORK/.claude/skills/gmr"; fi
fi

if [ "$ARM" = net ] && [ ! -d "$NET" ]; then
  python3 "$REPO/tools/net/render.py" --net "$NET" --repo "$WORK" --under crates/gmr-core >"$OUT/seed.json"
fi
if [ ! -f "$OUT/keys.txt" ]; then
  grep -ho '^@[A-Za-z]* [^ ]*#[^ =]*' "$NET"/*.md | awk '{print $2}' | sort -u >"$OUT/keys.txt"
fi

refresh() {
  while read -r k; do
    ( cd "$WORK" && "$GMR_BIN" observe "$k" >/dev/null 2>&1 ) || true
  done <"$OUT/keys.txt"
}

refresh

task_text="$(awk -v n="$TASK" '/^## Task /{p=($3==n)} p' "$HERE/tasks.md")"
common="You are working in this repository on task $TASK of a sequence of ten. Do the task and make \`cargo test -p gmr-core\` pass. Do not commit. Repository rules: zero comments in code; a bug gets a test first; keep changes inside crates/gmr-core unless the task says otherwise."
if [ "$ARM" = net ]; then
  card="$(sed "s#GMRBIN#$GMR_BIN#g; s#NET#$NET#g; s#TASK#t$TASK#g" "$HERE/entry-card.md")"
  preamble="$card

Before reading any code, walk every function, type or file you expect to touch. When the task is done, write back what fills a slot: a purpose or contract a later agent would need, and a Claim for any conclusion you reached about this code."
else
  preamble="This repository is managed by gmr; read .claude/skills/gmr/SKILL.md first and use it the way it says, before reading code and when you finish. The gmr binary is at $GMR_BIN."
fi

if command -v timeout >/dev/null 2>&1; then LIMIT=(timeout "$TIMEOUT")
elif command -v gtimeout >/dev/null 2>&1; then LIMIT=(gtimeout "$TIMEOUT")
else LIMIT=(perl -e 'alarm shift; exec @ARGV' "$TIMEOUT"); fi

( cd "$WORK" && "${LIMIT[@]}" "$CLAUDE_BIN" -p "$preamble

$common

$task_text" --model "$MODEL" --allowedTools Bash Read Grep Glob Edit Write MultiEdit \
  --output-format stream-json --verbose ) >"$OUT/$RUN.stream.jsonl" 2>"$OUT/$RUN.stderr.txt" || true

if [ ! -s "$OUT/$RUN.stream.jsonl" ]; then
  echo "$RUN: claude produced no transcript; see $OUT/$RUN.stderr.txt" >&2
  exit 3
fi

tests=0
( cd "$WORK" && cargo test -p gmr-core -q >"$OUT/$RUN.tests.txt" 2>&1 ) && tests=1
if [ "$tests" = 1 ]; then
  git -C "$WORK" add -A
  git -C "$WORK" "${GIT_ID[@]}" commit -qm "task $TASK" || true
else
  git -C "$WORK" checkout -q -- .
  git -C "$WORK" clean -qfd
fi

refresh

stale=null; landed=null; attempted=null
if [ "$ARM" = net ]; then
  stale="$(python3 "$REPO/tools/net/check.py" --net "$NET" --repo "$WORK" --count 2>/dev/null || true)"
  read -r landed attempted < <(python3 - "$NET/writes.jsonl" "t$TASK" <<'PY'
import json, sys, os
path, task = sys.argv[1], sys.argv[2]
landed = attempted = 0
if os.path.exists(path):
    for line in open(path):
        try: r = json.loads(line)
        except ValueError: continue
        if r.get("task") == task:
            attempted += 1
            landed += r.get("exit") in (0, 1)
print(landed, attempted)
PY
)
fi

meter="$(python3 "$HERE/meter.py" "$OUT/$RUN.stream.jsonl" "$ARM" "$GMR_BIN" "$NET")"
printf '{"arm":"%s","task":%s,"tests_pass":%s,"writes_landed":%s,"writes_attempted":%s,"stale":%s,"meter":%s}\n' \
  "$ARM" "$TASK" "$([ "$tests" = 1 ] && echo true || echo false)" "$landed" "$attempted" "${stale:-null}" "$meter" \
  >>"$OUT/results.jsonl"
tail -1 "$OUT/results.jsonl"
