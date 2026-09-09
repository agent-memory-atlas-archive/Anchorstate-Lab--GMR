"""Seed a Phase 0 net from migration-full.md, design.md §13 and §14.

    python3 tools/net/render.py --net NET --repo . --under crates/gmr-core

For every note whose `about:` names a coordinate under --under, the note's row in
docs/ontology/migration-full.md becomes one container: its Decisions and Policies as
nodes (claim from the row, rationale from the note's first paragraph, the review
question from its closing section, valid_from from git), and the row's edges, except
readings and must_change_with, which needs a reason nobody wrote down. Every Decision
rests on the current fact address of each `about:` coordinate. The source is
`reviewed git`: the owner committed every note.
"""

import argparse
import json
import os
import re
import subprocess

import grammar
import net as netmod

ROW = re.compile(r"^\| (\S+\.md) \| (fits|retired|new_slot) \| (.*?) \| (.*?) \| (.*?) \| (.*?) \|$")
KIND = {"D": "Decision", "Decision": "Decision", "P": "Policy", "Policy": "Policy"}
READINGS = {"calls", "imports", "contains", "implements", "reads", "exposes"}
SKIPPED = {"must_change_with"}


def frontmatter(text: str):
    if not text.startswith("---"):
        return {}, text
    end = text.index("\n---", 3)
    head, body = text[3:end], text[end + 4 :]
    about, watch = [], []
    mode = None
    for line in head.splitlines():
        if line.startswith("about:"):
            rest = line[6:].strip().strip('"')
            about = [rest] if rest else []
            mode = "about"
        elif line.startswith("watch:"):
            watch = re.findall(r"[a-z]+", line[6:])
            mode = None
        elif line.startswith("  - ") and mode == "about":
            about.append(line[4:].strip().strip('"'))
        elif not line.startswith(" "):
            mode = None
    return {"about": about, "watch": watch}, body


def first_paragraph(body: str) -> str:
    lines = body.strip().splitlines()
    i = 0
    while i < len(lines) and (not lines[i].strip() or lines[i].startswith("#")):
        i += 1
    para = []
    while i < len(lines) and lines[i].strip() and not lines[i].startswith("#") and not lines[i].startswith("```"):
        para.append(lines[i].strip())
        i += 1
    text = " ".join(para)
    text = re.sub(r"\[\[([^\]]+)\]\]", r"\1", text)
    text = re.sub(r"\*\*([^*]+)\*\*", r"\1", text)
    return text.strip()


def ask_of(body: str) -> str | None:
    m = re.search(r"^## .*ask.*$", body, re.M | re.I)
    if not m:
        return None
    rest = body[m.end() :].strip()
    para = first_paragraph("# x\n\n" + rest)
    sentence = re.split(r"(?<=[.?!])\s", para, maxsplit=1)[0]
    return sentence if sentence.endswith(("?", ".")) else None


def created_on(repo: str, note: str) -> str | None:
    r = subprocess.run(
        ["git", "log", "--diff-filter=A", "--format=%ad", "--date=short", "--", f"memories/{note}"],
        cwd=repo,
        capture_output=True,
        text=True,
    )
    lines = [l for l in r.stdout.splitlines() if l.strip()]
    return lines[-1] if lines else None


def split_decisions(cell: str):
    items = re.split(r";\s+(?=[DP]: )", cell.strip())
    out = []
    for item in items:
        m = re.match(r"^([DP]): (.*)$", item.strip())
        if m:
            out.append((KIND[m.group(1)], m.group(2).strip()))
    return out


def split_edges(cell: str):
    out = []
    i = 0
    while i < len(cell):
        m = re.match(r"\s*,?\s*(\w+)\(", cell[i:])
        if not m:
            break
        kind = m.group(1)
        j = i + m.end()
        depth = 1
        k = j
        while k < len(cell) and depth:
            depth += cell[k] == "(" and 1 or (cell[k] == ")" and -1 or 0)
            k += 1
        inner = cell[j : k - 1]
        if " → " in inner:
            frm, to = inner.split(" → ", 1)
            out.append((kind, frm.strip(), to.strip()))
        i = k
    return out


KINDS = {"function": "Function", "type": "Type", "struct": "Type", "enum": "Type", "trait": "Type", "module": "Module", "field": "Field"}


def entity_type(key: str, kind: str | None) -> str:
    if "#" not in key:
        return "File"
    return KINDS.get(kind or "", "Function")


def normalise(repo: str, ref: str):
    if ":" not in ref:
        return None, ref
    type_, key = ref.split(":", 1)
    type_ = KIND.get(type_, type_)
    if "/" in key and not os.path.exists(os.path.join(repo, key.split("#", 1)[0])):
        path, _, rest = key.partition("#")
        crate, _, file = path.partition("/")
        for candidate in (f"crates/{crate}/src/{file}", f"batteries/{crate.removeprefix('gmr-')}/src/{file}", f"crates/{crate}/{file}"):
            if os.path.exists(os.path.join(repo, candidate)):
                key = candidate + ("#" + rest if rest else "")
                break
    return type_, key


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--net", required=True)
    ap.add_argument("--repo", default=".")
    ap.add_argument("--under", required=True)
    ap.add_argument("--migration", default="docs/ontology/migration-full.md")
    ap.add_argument("--notes", default="memories")
    args = ap.parse_args()
    os.makedirs(args.net, exist_ok=True)
    n = netmod.Net(args.net, args.repo)
    rows = {}
    for line in open(os.path.join(args.repo, args.migration), encoding="utf-8"):
        m = ROW.match(line.rstrip("\n"))
        if m:
            rows[m.group(1)] = m.groups()
    summary = {"notes": 0, "decisions": 0, "policies": 0, "edges": 0, "skipped_edges": {}, "unaddressed": []}
    for note in sorted(os.listdir(os.path.join(args.repo, args.notes))):
        if not note.endswith(".md") or note == "README.md":
            continue
        text = open(os.path.join(args.repo, args.notes, note), encoding="utf-8").read()
        meta, body = frontmatter(text)
        if not any(a.startswith(args.under) for a in meta.get("about", [])):
            continue
        row = rows.get(note)
        if not row:
            continue
        _, fit, decisions_cell, edges_cell, _, _ = row
        summary["notes"] += 1
        decisions = split_decisions(decisions_cell)
        edges = split_edges(edges_cell)
        ids = []
        for _, frm, to in edges:
            for ref in (frm, to):
                t, k = normalise(args.repo, ref)
                if t in ("Decision", "Policy") and (t, k) not in ids:
                    ids.append((t, k))
        stem = note[:-3]
        assigned = []
        used = set()
        for i, (kind, _) in enumerate(decisions, 1):
            pick = next((x for x in ids if x[0] == kind and x[1].endswith(f"-{i}") and x not in used), None)
            if pick is None:
                pick = next((x for x in ids if x[0] == kind and x not in used), None)
            if pick is None:
                pick = (kind, f"{stem.lower()}-{i}")
            used.add(pick)
            assigned.append(pick)
        rests_on = []
        abouts = []
        for about in meta.get("about", []):
            now = n.address_of(about)
            if now and now.get("addr"):
                rests_on.append({"key": about, "addr": now["addr"], "paths": []})
                abouts.append((entity_type(about, now.get("kind")), about))
            else:
                summary["unaddressed"].append(about)
        rationale = first_paragraph(body)
        ask = ask_of(body)
        born = created_on(args.repo, note)
        out = [f"# {stem}"]
        for (kind, id_), (_, claim) in zip(assigned, decisions):
            out.append("")
            out.append(grammar.render_node(kind, id_))
            out.append(grammar.render_slot("claim", claim if claim.endswith((".", "?", "!")) else claim + ".", "h"))
            if rationale:
                out.append(grammar.render_slot("rationale", rationale, "h"))
            if ask:
                out.append(grammar.render_slot("review_question", ask, "h"))
            if born:
                out.append(grammar.render_slot("valid_from", born, "h"))
            out.append(grammar.render_slot("standing", "retired" if fit == "retired" else "live", "h"))
            summary["decisions" if kind == "Decision" else "policies"] += 1
        by_from = {}
        for kind, id_ in assigned:
            rel = "decided_by" if kind == "Decision" else "governed_by"
            for et, ek in abouts:
                if not any(k == rel and normalise(args.repo, to) == (kind, id_) for k, _, to in edges):
                    by_from.setdefault((et, ek), []).append((rel, kind, id_))
        for kind, frm, to in edges:
            if kind in READINGS or kind in SKIPPED:
                summary["skipped_edges"][kind] = summary["skipped_edges"].get(kind, 0) + 1
                continue
            ft, fk = normalise(args.repo, frm)
            tt, tk = normalise(args.repo, to)
            if ft is None or tt is None:
                summary["skipped_edges"]["untyped"] = summary["skipped_edges"].get("untyped", 0) + 1
                continue
            by_from.setdefault((ft, fk), []).append((kind, tt, tk))
        for (ft, fk), lst in by_from.items():
            out.append("")
            out.append(grammar.render_node(ft, fk))
            seen = set()
            for kind, tt, tk in lst:
                if (kind, tt, tk) in seen:
                    continue
                seen.add((kind, tt, tk))
                out.append(grammar.render_edge(">", kind, tt, tk, "h"))
                summary["edges"] += 1
        out.append("")
        out.append(grammar.render_footer("h", "human", "zongming", "reviewed", "git", rests_on, short=False))
        with open(os.path.join(args.net, note), "w", encoding="utf-8") as f:
            f.write("\n".join(out) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
