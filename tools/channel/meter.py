"""Per-tool-call byte accounting over a claude -p stream-json transcript.

Every tool result's bytes are classified by where the information came from:
the gmr channel, source files, data files, memory notes, the skill doc, or
other. The result event contributes usage tokens, turns, cost, duration and
permission denials. V is measured on this side - bytes that entered the
context - independent of API billing.
"""

import json
import sys


def classify(name: str, tool_input: dict, gmr_bin: str) -> str:
    if name == "Bash":
        cmd = tool_input.get("command", "")
        if gmr_bin in cmd or cmd.strip().startswith("gmr ") or " gmr " in cmd:
            return "gmr"
        for cls, marker in (
            ("src", "src/"),
            ("data", "data/"),
            ("memory", "memories/"),
            ("skill", ".claude"),
        ):
            if marker in cmd:
                return cls
        return "other"
    path = str(
        tool_input.get("file_path")
        or tool_input.get("path")
        or tool_input.get("pattern", "")
    )
    if ".claude" in path:
        return "skill"
    for cls, marker in (("src", "src/"), ("data", "data/"), ("memory", "memories/")):
        if marker in path:
            return cls
    if name in ("Grep", "Glob"):
        return "search"
    return "other"


def content_bytes(content) -> int:
    if isinstance(content, str):
        return len(content.encode())
    if isinstance(content, list):
        return sum(
            len(str(part.get("text", "")).encode())
            for part in content
            if isinstance(part, dict)
        )
    return 0


def main():
    gmr_bin = sys.argv[2] if len(sys.argv) > 2 else "/gmr"
    pending = {}
    by_class = {}
    calls = {}
    summary = {}
    for line in open(sys.argv[1]):
        line = line.strip()
        if not line:
            continue
        try:
            event = json.loads(line)
        except ValueError:
            continue
        kind = event.get("type")
        message = event.get("message")
        if not isinstance(message, dict):
            message = {}
        content = message.get("content")
        if not isinstance(content, list):
            content = []
        for part in content:
            if not isinstance(part, dict):
                continue
            if part.get("type") == "tool_use":
                cls = classify(part.get("name", ""), part.get("input") or {}, gmr_bin)
                pending[part.get("id")] = cls
                calls[cls] = calls.get(cls, 0) + 1
            if part.get("type") == "tool_result":
                cls = pending.pop(part.get("tool_use_id"), "other")
                by_class[cls] = by_class.get(cls, 0) + content_bytes(part.get("content"))
        if kind == "result":
            usage = event.get("usage") or {}
            summary = {
                "num_turns": event.get("num_turns"),
                "duration_ms": event.get("duration_ms"),
                "total_cost_usd": event.get("total_cost_usd"),
                "input_tokens": usage.get("input_tokens"),
                "cache_creation_input_tokens": usage.get("cache_creation_input_tokens"),
                "cache_read_input_tokens": usage.get("cache_read_input_tokens"),
                "output_tokens": usage.get("output_tokens"),
                "permission_denials": len(event.get("permission_denials") or []),
            }
    total = sum(by_class.values())
    truth = by_class.get("gmr", 0) + by_class.get("memory", 0)
    print(
        json.dumps(
            {
                "bytes_by_class": by_class,
                "calls_by_class": calls,
                "V_total_bytes": total,
                "P_gmr_share": round((truth / total), 3) if total else None,
                **summary,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
