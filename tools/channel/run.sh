#!/usr/bin/env bash
# Channel comparison: completeness and volume of the information that reaches
# the agent through gmr vs enumeration. ARM=a1|a2|a3|a1p SCALE=15|60 TAG=r1
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
GMR_BIN="${GMR_BIN:-$REPO/target/release/gmr}"
CLAUDE_BIN="${CLAUDE_BIN:-claude}"
ARM="${ARM:?a1|a2|a3|a1p}"
SCALE="${SCALE:-15}"
TAG="${TAG:-r1}"
OUT="${OUT:-$REPO/.anchor/output/channel}"
BASE15="${BASE15:-/tmp/channel-base15/repo}"
BASE60="${BASE60:-/tmp/channel-base60/repo}"
MODEL_ARGS=()
[ -n "${MODEL:-}" ] && MODEL_ARGS=(--model "$MODEL")
mkdir -p "$OUT"

BASE="$BASE15"; [ "$SCALE" = "60" ] && BASE="$BASE60"
RUN="channel-$ARM-$SCALE-${MODEL:-sonnet}-$TAG"
DIR="$(mktemp -d)/repo"
cp -R "$BASE" "$DIR"

json_tail='End your reply with one JSON object: {"affected": ["..."], "constraints": ["..."], "relied_on": ["..."], "verbs_used": ["..."]}'
task="Change request for crates/exporter: every ledger line written by write_entry should gain a fourth field carrying an ISO currency code. Before any implementation, answer: what constrains this change, and who or what would be affected if the line shape changes? Do not modify any file. $json_tail"

gmr_preamble="This repository is managed by gmr; read .claude/skills/gmr/SKILL.md first. The gmr binary is at $GMR_BIN. Cite records by the addresses gmr prints. Do not commit or push."
code_preamble="Investigate the repository as needed. Cite the files you relied on by path. Do not commit or push."

TOOLS=(--allowedTools Bash Read Grep Glob)
PREAMBLE="$gmr_preamble"
case "$ARM" in
  a1)
    TOOLS=(--allowedTools "Bash($GMR_BIN *)" "Read(.claude/**)")
    PREAMBLE="$gmr_preamble Only gmr gives you information here: other commands and file reads are not permitted."
    ;;
  a2)
    rm -rf "$DIR/.anchor" "$DIR/memories" "$DIR/.claude" "$DIR/.git"
    ( cd "$DIR" && git init -q && git config user.email x@example.invalid \
      && git config user.name x && git add -A && git commit -qm code )
    PREAMBLE="$code_preamble"
    ;;
  a3) ;;
  a1p)
    rm -rf "$DIR/src" "$DIR/data" "$DIR/crates"
    ;;
esac

( cd "$DIR" && "$CLAUDE_BIN" ${MODEL_ARGS[@]+"${MODEL_ARGS[@]}"} -p "$PREAMBLE

Task: $task" "${TOOLS[@]}" --output-format stream-json --verbose ) \
  >"$OUT/$RUN.stream.jsonl" 2>"$OUT/$RUN.stderr.txt" || true

python3 - "$OUT/$RUN.stream.jsonl" >"$OUT/$RUN.final.txt" <<'PY'
import json, sys
text = ""
for line in open(sys.argv[1]):
    try: event = json.loads(line)
    except ValueError: continue
    if event.get("type") == "result":
        text = event.get("result") or ""
print(text)
PY

{
  echo "{\"run\": \"$RUN\","
  echo "\"meter\":"; python3 "$HERE/meter.py" "$OUT/$RUN.stream.jsonl" "$GMR_BIN"
  echo ",\"facts\":"; python3 "$HERE/facts.py" "$OUT/$RUN.final.txt" || echo '{"error": "no json"}'
  echo "}"
} | python3 -c "import json,sys; print(json.dumps(json.loads(sys.stdin.read())))" >>"$OUT/results.jsonl"
tail -1 "$OUT/results.jsonl" | python3 -m json.tool | head -30
