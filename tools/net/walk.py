"""Phase 0 read entry: one node and one hop, in the line grammar, at most 4 KB.

    python3 tools/net/walk.py --net NET --task T <label|Type:key> [--stale include|exclude|only]

Every address printed is recorded in NET/issued/<task>.txt so a later write can cite it.
"""

import argparse
import sys

import grammar
import net as netmod

BUDGET = 4096


def stale_of(n: netmod.Net, record):
    marks = []
    for r in record.rests_on:
        now = n.address_of(r["key"])
        if now is None or not now.get("addr"):
            continue
        if not now["addr"].startswith(r["addr"][:8]) and not r["addr"].startswith(now["addr"][:8]):
            marks.append(f"{r['key']}@{now['addr'][:8]}")
    return marks


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--net", required=True)
    ap.add_argument("--repo", default=".")
    ap.add_argument("--task", required=True)
    ap.add_argument("--stale", choices=["include", "exclude", "only"], default="include")
    ap.add_argument("--budget", type=int, default=BUDGET)
    ap.add_argument("label")
    args = ap.parse_args()
    n = netmod.Net(args.net, args.repo)
    hits = n.find(args.label)
    if not hits:
        print(f"! {args.label} is not in the net")
        sys.exit(1)
    if len(hits) > 1:
        for h in hits[:5]:
            print(h)
        sys.exit(1)
    subject = hits[0]
    node = n.nodes[subject]
    tags = {}
    footers = []

    def tag_for(record, stale):
        sig = (record.who, record.author, record.source, record.basis, tuple((r["key"], r["addr"][:8], tuple(r["paths"])) for r in record.rests_on), tuple(stale))
        if sig not in tags:
            tags[sig] = chr(ord("a") + len(tags))
            footers.append(grammar.render_footer(tags[sig], record.who, record.author, record.source, record.basis, record.rests_on, " ".join(stale) or None))
        return tags[sig]

    lines = []
    addr = None
    if "#" in node["key"] or "/" in node["key"]:
        now = n.address_of(node["key"])
        if now and now.get("addr"):
            addr = now["addr"]
            n.issue(args.task, node["key"], addr)
    lines.append(grammar.render_node(node["type"], node["key"], addr))
    shown = 0
    for r in n.slots_of(subject):
        stale = stale_of(n, r)
        if args.stale == "exclude" and stale or args.stale == "only" and not stale:
            continue
        lines.append(grammar.render_slot(r.name, r.value, tag_for(r, stale), r.hash, bool(stale)))
        shown += 1
    edge_lines = []
    for r in n.edges_out(subject):
        stale = stale_of(n, r)
        if args.stale == "exclude" and stale or args.stale == "only" and not stale:
            continue
        t, k = r.value.split(":", 1)
        edge_lines.append(grammar.render_edge(">", r.name, t, k, tag_for(r, stale), r.hash, r.extra.get("reason")))
        for far in n.slots_of(r.value):
            if far.name in ("claim", "review_question", "definition", "text"):
                label = "ask" if far.name == "review_question" else far.name
                edge_lines.append(f"  {label}: {far.value}")
    for r in n.edges_in(subject):
        stale = stale_of(n, r)
        if args.stale == "exclude" and stale or args.stale == "only" and not stale:
            continue
        t, k = r.subject.split(":", 1)
        edge_lines.append(grammar.render_edge("<", r.name, t, k, tag_for(r, stale), r.hash))
    body = "\n".join(lines)
    kept, dropped = [], 0
    for l in edge_lines:
        candidate = body + "\n" + "\n".join(kept + [l]) + "\n" + "\n".join(footers)
        if len(candidate.encode()) > args.budget and not l.startswith("  "):
            dropped += 1
            continue
        if dropped and l.startswith("  "):
            continue
        kept.append(l)
    out = [body] + kept
    if dropped:
        out.append(f"…+{dropped} edges")
    out += footers
    if len(n.slots_of(subject)) + len(n.edges_out(subject)) + len(n.edges_in(subject)) == 0:
        out.append("(nothing recorded here yet)")
    print("\n".join(out))


if __name__ == "__main__":
    main()
