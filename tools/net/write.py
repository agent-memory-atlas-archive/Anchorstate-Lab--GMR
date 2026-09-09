"""Phase 0 write: validate a line-grammar document, then land it in the net.

    python3 tools/net/write.py --net NET --task T [--dry-run] < doc

Exit 0 landed · 1 landed with warnings · 2 refused, nothing written. The reply is at most
200 bytes. Every attempt is appended to NET/writes.jsonl for the experiment's grading.
"""

import argparse
import json
import os
import sys
import time

import grammar
import net as netmod
import validate


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--net", required=True)
    ap.add_argument("--repo", default=".")
    ap.add_argument("--task", required=True)
    ap.add_argument("--ontology", default="packs/coding/ontology.yaml")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()
    text = sys.stdin.read()
    onto = validate.load_ontology(os.path.join(args.repo, args.ontology))
    n = netmod.Net(args.net, args.repo)
    doc = grammar.parse(text)
    errors, warnings = validate.validate(doc, onto, n, args.task, args.repo)
    attempt = {
        "task": args.task,
        "at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "bytes": len(text.encode()),
        "errors": [c for _, c, _ in errors],
        "warnings": [c for _, c, _ in warnings],
    }
    if errors:
        print(validate.report(errors, warnings))
        attempt["exit"] = 2
        log_attempt(n, attempt)
        sys.exit(2)
    if args.dry_run:
        print(validate.report([], warnings) or "+ ok (dry run)")
        sys.exit(0)

    reply = []
    edges = 0
    issued = n.issued(args.task)
    for block in doc.blocks:
        subject = block.id
        new_node = subject not in n.nodes or not n.slots_of(subject) and not n.edges_out(subject)
        container = n.container_for(subject)
        for op in block.ops:
            old = n.by_hash(op.hash)
            n.remove_line(old)
            n.log({"op": op.op, "task": args.task, "record": old.to_json()})
            if op.op == "retire":
                reply.append(f"- ^{old.hash[:8]} retired")
                continue
            f = doc.footers[op.tag]
            home = n.container_for(old.subject)
            tag = n.free_tag(home)
            if old.kind == "slot":
                line = grammar.render_slot(old.name, old.value, tag)
            else:
                t, k = old.value.split(":", 1)
                line = grammar.render_edge(">", old.name, t, k, tag, None, old.extra.get("reason"))
            n.append_lines(home, old.subject, [line], footer_line(f, tag, issued), tag)
            reply.append(f"~ ^{old.hash[:8]} attested")
        doc_tags = []
        for item in list(block.slots) + list(block.edges):
            if item.tag not in doc_tags:
                doc_tags.append(item.tag)
        assigned = {}
        for doc_tag in doc_tags:
            assigned[doc_tag] = n.free_tag(container, assigned.values())
        lines = []
        for slot in block.slots:
            if slot.replaces:
                old = n.by_hash(slot.replaces)
                n.remove_line(old)
                n.log({"op": "replace", "task": args.task, "record": old.to_json()})
            lines.append(grammar.render_slot(slot.name, slot.text, assigned[slot.tag]))
            addr = next((r["addr"][:8] for r in doc.footers[slot.tag].rests_on()), None)
            reply.append(f"+ {subject}.{slot.name}" + (f" @{addr}" if addr else ""))
        for edge in block.edges:
            if edge.replaces:
                old = n.by_hash(edge.replaces)
                n.remove_line(old)
                n.log({"op": "replace", "task": args.task, "record": old.to_json()})
            lines.append(grammar.render_edge(edge.direction, edge.rel, edge.type, edge.key, assigned[edge.tag], None, edge.reason))
            edges += 1
        if lines:
            for doc_tag, tag in assigned.items():
                n.append_lines(container, subject, [], footer_line(doc.footers[doc_tag], tag, issued), tag)
            n.append_lines(container, subject, lines, "", next(iter(assigned.values())))
            n.log({"op": "add", "task": args.task, "subject": subject, "lines": lines, "container": container})
            if new_node:
                reply.insert(0, f"+ {subject}")
    if edges:
        reply.append(f"+ {edges} edges")
    out = "\n".join(reply)
    warn_text = validate.report([], warnings)
    if warn_text:
        out = out + "\n" + warn_text
    if len(out.encode()) > 200:
        out = out.encode()[:199].decode(errors="ignore") + "…"
    print(out)
    attempt["exit"] = 1 if warnings else 0
    log_attempt(n, attempt)
    sys.exit(attempt["exit"])


def footer_line(f: grammar.Footer, tag: str, issued: dict | None = None) -> str:
    rests = f.rests_on()
    for r in rests:
        full = next((a for a in (issued or {}) if a.startswith(r["addr"])), None)
        if full:
            r["addr"] = full
    return grammar.render_footer(tag, f.who, f.name, f.how, f.basis(), rests, short=False)


def log_attempt(n: netmod.Net, attempt: dict):
    with open(os.path.join(n.root, "writes.jsonl"), "a", encoding="utf-8") as fh:
        fh.write(json.dumps(attempt, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
