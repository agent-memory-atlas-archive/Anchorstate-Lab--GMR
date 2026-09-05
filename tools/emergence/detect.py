"""Mechanically detects whether the memory layer holds any artifact spanning
the exporter and reporter modules: a note whose about covers both, a link whose
two ends live in different modules, or a recorded conclusion standing on
anchors from both. The code graph has no such edge by construction, so any hit
is the memory layer connecting what the fact layer does not.
"""

import json
import os
import pathlib
import re
import subprocess
import sys

from fixture import SEED_NOTES

A = "crates/exporter"
B = "crates/reporter"


def sh(cwd, *argv):
    out = subprocess.run(argv, cwd=cwd, capture_output=True, text=True)
    return out.stdout


def module_of(coordinate: str):
    if coordinate.startswith(A):
        return "A"
    if coordinate.startswith(B):
        return "B"
    return None


def spans(coordinates):
    hit = {module_of(c) for c in coordinates}
    return "A" in hit and "B" in hit


def note_coordinates(root: pathlib.Path, rel: str):
    text = (root / rel).read_text()
    match = re.match(r"^---\n(.*?)\n---", text, re.S)
    if not match:
        return [], []
    front = match.group(1)
    abouts = re.findall(r"^\s*(?:about:\s*|-\s*)[\"']?([^\s\"']+)[\"']?\s*$", front, re.M)
    linked = re.findall(r"^\s+[a-z-]+:\s*\[(.*?)\]", front, re.M)
    targets = [t.strip().strip("\"'") for group in linked for t in group.split(",") if t.strip()]
    return [a for a in abouts if "/" in a], targets


def main():
    root = pathlib.Path(sys.argv[1]).resolve()
    gmr = os.environ.get("GMR_BIN", "gmr")
    findings = []

    notes = sorted(
        str(p.relative_to(root))
        for p in (root / "memories").glob("*.md")
    )
    note_module = {}
    for rel in notes:
        abouts, targets = note_coordinates(root, rel)
        mods = {module_of(a) for a in abouts} - {None}
        note_module[pathlib.Path(rel).stem] = mods
        if spans(abouts):
            findings.append({"kind": "note-spans-both", "note": rel, "about": abouts})
    for rel in notes:
        abouts, targets = note_coordinates(root, rel)
        mine = {module_of(a) for a in abouts} - {None}
        for target in targets:
            theirs = note_module.get(target.split(":")[-1].split("/")[-1].removesuffix(".md"), set())
            if mine and theirs and mine != theirs and (mine | theirs) >= {"A", "B"}:
                findings.append(
                    {"kind": "frontmatter-link-across", "note": rel, "to": target}
                )

    ground = sh(root, gmr, "ground", "--json")
    try:
        stood = json.loads(ground)
    except ValueError:
        stood = []
    for one in stood:
        keys = [entry.get("key", "") for entry in one.get("on", [])]
        if spans(keys):
            findings.append(
                {
                    "kind": "conclusion-stands-on-both",
                    "claim": one.get("claim"),
                    "on": keys,
                    "text": (one.get("claim_text") or ""),
                }
            )

    asserted = []
    for rel in notes:
        name = pathlib.Path(rel).stem
        links = sh(root, gmr, "links", f"memories/{pathlib.Path(rel).name}", "--json")
        try:
            held = json.loads(links)
        except ValueError:
            continue
        for edge in held.get("out", []):
            asserted.append((rel, edge))
    for rel, edge in asserted:
        mine = note_module.get(pathlib.Path(rel).stem, set())
        to_id = edge.get("to", {}).get("external_id", "")
        theirs = note_module.get(pathlib.Path(to_id).stem, set())
        if mine and theirs and (mine | theirs) >= {"A", "B"} and mine != theirs:
            findings.append({"kind": "asserted-link-across", "from": rel, "edge": edge})

    new_notes = [n for n in notes if n not in SEED_NOTES]
    print(
        json.dumps(
            {
                "cross_edge": bool(findings),
                "findings": findings,
                "new_notes": new_notes,
                "notes_seen": notes,
            },
            indent=2,
        )
    )
    sys.exit(0)


if __name__ == "__main__":
    main()
