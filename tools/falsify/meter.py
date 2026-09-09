"""Bytes that entered the context outside the memory channel before the first edit.

    python3 meter.py <stream.jsonl> <arm: net|ctl> <gmr_bin> <net_dir>

Classes: channel (walk/write/check for net; gmr and SKILL.md for ctl), edit (the first
Edit/Write/MultiEdit or a Bash that writes a file), outside (everything else). The
number that matters is outside bytes among tool results whose call came before the
first edit; the totals and the result event's usage ride along.
"""

import json
import re
import sys

WRITES = re.compile(r"(^|[;&|]\s*)(sed -i|tee |cat\s*>|>\s*[^&\s]|cp |mv |git apply|patch )")


def classify(name, tool_input, arm, gmr_bin, net_dir):
    if name in ("Edit", "Write", "MultiEdit", "NotebookEdit"):
        return "edit"
    if name == "Bash":
        cmd = tool_input.get("command", "")
        if arm == "net" and "tools/net/" in cmd:
            return "channel"
        if arm == "ctl" and (gmr_bin in cmd or re.search(r"(^|\s)gmr\s", cmd)):
            return "channel"
        if WRITES.search(cmd):
            return "edit"
        return "outside"
    path = str(tool_input.get("file_path") or tool_input.get("path") or tool_input.get("pattern") or "")
    if arm == "ctl" and ".claude/skills" in path:
        return "channel"
    if arm == "net" and net_dir and net_dir in path:
        return "channel"
    return "outside"


def content_bytes(content):
    if isinstance(content, str):
        return len(content.encode())
    if isinstance(content, list):
        return sum(len(str(p.get("text", "")).encode()) for p in content if isinstance(p, dict))
    return 0


def main():
    stream, arm, gmr_bin, net_dir = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
    pending = {}
    before = {}
    total = {}
    calls = {}
    edited = False
    summary = {}
    write_verbs = 0
    for line in open(stream, encoding="utf-8"):
        line = line.strip()
        if not line:
            continue
        try:
            event = json.loads(line)
        except ValueError:
            continue
        message = event.get("message") if isinstance(event.get("message"), dict) else {}
        for part in message.get("content") or []:
            if not isinstance(part, dict):
                continue
            if part.get("type") == "tool_use":
                inp = part.get("input") or {}
                cls = classify(part.get("name", ""), inp, arm, gmr_bin, net_dir)
                if cls == "edit":
                    edited = True
                if part.get("name") == "Bash":
                    cmd = inp.get("command", "")
                    if arm == "ctl" and re.search(r"gmr\s+(said|anchor|bind|link|reaffirm|condense|attest|cobound)\b", cmd):
                        write_verbs += 1
                    if arm == "net" and "tools/net/write.py" in cmd:
                        write_verbs += 1
                pending[part.get("id")] = (cls, not edited)
                calls[cls] = calls.get(cls, 0) + 1
            if part.get("type") == "tool_result":
                cls, early = pending.pop(part.get("tool_use_id"), ("outside", False))
                n = content_bytes(part.get("content"))
                total[cls] = total.get(cls, 0) + n
                if early:
                    before[cls] = before.get(cls, 0) + n
        if event.get("type") == "result":
            usage = event.get("usage") or {}
            summary = {
                "num_turns": event.get("num_turns"),
                "duration_ms": event.get("duration_ms"),
                "total_cost_usd": event.get("total_cost_usd"),
                "input_tokens": usage.get("input_tokens"),
                "cache_read_input_tokens": usage.get("cache_read_input_tokens"),
                "output_tokens": usage.get("output_tokens"),
            }
    print(
        json.dumps(
            {
                "outside_before_edit": before.get("outside", 0),
                "channel_before_edit": before.get("channel", 0),
                "bytes_total": total,
                "calls": calls,
                "write_verbs": write_verbs,
                **summary,
            }
        )
    )


if __name__ == "__main__":
    main()
