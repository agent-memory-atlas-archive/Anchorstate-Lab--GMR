"""The write-time validator, design.md §7. Refusals name the line and the code.

E01 type declared · E02 key form · E03 fact-keyed entity resolves now · E04 slot declared
E05 required slots on creation · E06 source legal for the slot · E07 observed cites an
issued address · E08 one sentence · E09 relation kind exists and is not a reading
E10 endpoint types, must_change_with reason, supersedes target · E11 identity and CAS
E12 permission.  N01 history words · N02 dates and units · N03 prose cross-references
N04 restated source · N05 diary words · N06 unresolved identifier (warning) · N07 rationale
in claim · N08 near duplicate.
"""

import os
import re
import sys

import yaml

import grammar

HISTORY = re.compile(r"\b(used to|once|previously|no longer|briefly|originally|has not changed)\b", re.I)
DATE = re.compile(r"\b\d{4}-\d{2}-\d{2}\b")
UNITS = re.compile(r"\b\d+ ?(ms|KB|MB|GB|%|of \d+)\b")
CROSSREF = re.compile(r"\[\[|\]\(|\bsee \b|\bas \S+ says\b", re.I)
DIARY = re.compile(r"\b(we|I|probably|seems)\b")
RATIONALE = re.compile(r"\b(because|so that|since)\b", re.I)
BACKTICK = re.compile(r"`([^`]+)`")
TERMINATORS = ".!?。！？"


def load_ontology(path: str) -> dict:
    with open(path, encoding="utf-8") as f:
        return yaml.safe_load(f)


def key_form(onto: dict, type_: str, key: str) -> bool:
    form = onto["entities"][type_]["key"]
    pattern = onto["keys"].get(form)
    return bool(pattern and re.match(pattern, key))


def resolves(repo: str, type_: str, key: str, net) -> bool:
    if "#" not in key:
        return os.path.exists(os.path.join(repo, key))
    path, name = key.split("#", 1)
    if not os.path.isfile(os.path.join(repo, path)):
        return False
    found = net.address_of(key) if net else None
    if found and found.get("found"):
        return True
    leaf = re.split(r"[.:]", name)[-1] or name
    if not re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", leaf):
        return False
    try:
        text = open(os.path.join(repo, path), encoding="utf-8", errors="replace").read()
    except OSError:
        return False
    return re.search(rf"\b{re.escape(leaf)}\b", text) is not None


def one_sentence(text: str) -> bool:
    if len(text) > 240:
        return False
    inner = re.sub(r"`[^`]*`", "", text)
    ends = sum(inner.count(t) for t in TERMINATORS)
    return ends == 1 and text[-1] in TERMINATORS


def stems(text: str):
    out = set()
    for w in re.findall(r"[a-z]+", re.sub(r"`[^`]*`", "", text.lower())):
        for suffix in ("ing", "ed", "es", "s"):
            if w.endswith(suffix) and len(w) > len(suffix) + 2:
                w = w[: -len(suffix)]
                break
        out.add(w)
    return out


def jaccard(a: set, b: set) -> float:
    if not a or not b:
        return 0.0
    return len(a & b) / len(a | b)


def entity_source(repo: str, key: str) -> str:
    path = key.split("#", 1)[0]
    try:
        return open(os.path.join(repo, path), encoding="utf-8", errors="replace").read()
    except OSError:
        return ""


def validate(doc: grammar.Document, onto: dict, net, task: str, repo: str):
    errors, warnings = [], []
    err = lambda line, code, msg: errors.append((line, code, msg))
    warn = lambda line, code, msg: warnings.append((line, code, msg))
    for line, code, msg in doc.problems:
        err(line, code, msg)
    entities, relations = onto["entities"], onto["relations"]
    readings = set(onto.get("readings", []))
    issued = net.issued(task) if net else {}
    humans = set(onto.get("human", []))

    def footer_for(tag, line, what):
        if tag is None:
            err(line, "E06", f"{what} names no source tag")
            return None
        f = doc.footers.get(tag)
        if f is None:
            err(line, "E06", f"~{tag} has no footer")
            return None
        if f.how not in onto["sources"]:
            err(f.line, "E06", f"`{f.how}` is not a source class")
            return None
        if f.who == "human" and humans and f.name not in humans:
            warn(f.line, "W01", f"`{f.name}` is not on the human roster")
        return f

    def cited_ok(f):
        for r in f.rests_on():
            full = next((a for a in issued if a.startswith(r["addr"])), None)
            if full is None:
                err(f.line, "E07", f"{r['key']}@{r['addr'][:8]} was not handed out in task {task}")
                return False
            r["addr"] = full
        return True

    def type_ok(type_, line):
        if type_ not in entities:
            err(line, "E01", f"`{type_}` is not a type in the ontology")
            return False
        return True

    for block in doc.blocks:
        if not type_ok(block.type, block.line):
            continue
        spec = entities[block.type]
        if not key_form(onto, block.type, block.key):
            err(block.line, "E02", f"`{block.key}` is not a {spec['key']}")
            continue
        if spec.get("probe") and not resolves(repo, block.type, block.key, net):
            err(block.line, "E03", f"`{block.key}` does not resolve now; unresolved")
            continue
        existing = {r.name: r for r in (net.slots_of(block.id) if net else [])}
        creating = not existing
        slots_here = {s.name for s in block.slots}
        if creating and block.slots:
            for name, sspec in spec.get("slots", {}).items():
                if sspec.get("required") and name not in slots_here:
                    err(block.line, "E05", f"{block.type} needs `{name}`")
        if spec.get("author") == "agent" and block.slots:
            f = doc.footers.get(block.slots[0].tag) if block.slots[0].tag else None
            if f is not None and f.who != "agent":
                err(block.line, "E06", "a Claim is written by an agent")
            if f is not None and spec.get("rests_on") == "required" and not f.rests_on():
                err(f.line, "E06", "a Claim rests on something")
            if f is not None and spec.get("task") == "required" and f.name != task:
                err(f.line, "E06", f"a Claim's author is its task, `{task}`")
        for slot in block.slots:
            sspec = spec.get("slots", {}).get(slot.name)
            if sspec is None:
                err(slot.line, "E04", f"`{slot.name}` is not a slot of {block.type}")
                continue
            if sspec.get("values") and slot.text.rstrip(".") not in sspec["values"]:
                err(slot.line, "E04", f"`{slot.name}` is one of {', '.join(sspec['values'])}")
                continue
            f = footer_for(slot.tag, slot.line, f"`{slot.name}`")
            if f is None:
                continue
            if f.how not in sspec.get("sources", []):
                err(slot.line, "E06", f"`{slot.name}` does not accept `{f.how}`")
            if sspec.get("human") and f.who != "human":
                err(slot.line, "E06", f"`{slot.name}` is a person's judgment")
            if f.how == "observed" and not cited_ok(f):
                continue
            if sspec.get("sentence", True) and not sspec.get("values") and not sspec.get("date") and not one_sentence(slot.text):
                err(slot.line, "E08", "one sentence, one terminator, at most 240 characters")
            if slot.replaces:
                old = net.by_hash(slot.replaces) if net else None
                if old is None or old.subject != block.id or old.name != slot.name:
                    err(slot.line, "E11", f"^{slot.replaces[:8]} is not the current `{slot.name}` here")
                elif old.source == "decided" and f.who != "human":
                    err(slot.line, "E12", "a decided value is replaced by a person")
            elif slot.name in existing:
                err(slot.line, "E11", f"`{slot.name}` exists; replace it with ^{existing[slot.name].hash[:8]}")
            if sspec.get("sentence", True) and not sspec.get("values") and not sspec.get("date"):
                noise(slot, block, sspec, existing, repo, net, err, warn)
        for edge in block.edges:
            rspec = relations.get(edge.rel)
            if edge.rel in readings:
                err(edge.line, "E09", f"`{edge.rel}` is a reading; the probe writes it")
                continue
            if rspec is None:
                err(edge.line, "E09", f"`{edge.rel}` is not a relation")
                continue
            if not type_ok(edge.type, edge.line):
                continue
            if not key_form(onto, edge.type, edge.key):
                err(edge.line, "E02", f"`{edge.key}` is not a {entities[edge.type]['key']}")
                continue
            frm_t, to_t = (block.type, edge.type) if edge.direction == ">" else (edge.type, block.type)
            if not endpoint_ok(rspec["from"], frm_t, frm_t) or not endpoint_ok(rspec["to"], to_t, frm_t):
                err(edge.line, "E10", f"{edge.rel} does not run {frm_t} -> {to_t}")
                continue
            if rspec.get("reason") == "required" and not edge.reason:
                err(edge.line, "E10", f"{edge.rel} needs a reason line")
            if edge.rel == "supersedes" and net and f"{edge.type}:{edge.key}" not in net.nodes:
                err(edge.line, "E10", "supersedes names a target that is not in the net")
            f = footer_for(edge.tag, edge.line, f">{edge.rel}")
            if f is None:
                continue
            if f.how == "observed" and not cited_ok(f):
                continue
            frm, to = (block.id, f"{edge.type}:{edge.key}") if edge.direction == ">" else (f"{edge.type}:{edge.key}", block.id)
            dup = [r for r in (net.edges_out(frm) if net else []) if r.name == edge.rel and r.value == to]
            if edge.replaces:
                old = net.by_hash(edge.replaces) if net else None
                if old is None or old.kind != "edge":
                    err(edge.line, "E11", f"^{edge.replaces[:8]} is not an edge here")
            elif dup:
                err(edge.line, "E11", f"{edge.rel} to {to} exists ^{dup[0].hash[:8]}")
        for op in block.ops:
            old = net.by_hash(op.hash) if net else None
            if old is None:
                err(op.line, "E11", f"^{op.hash[:8]} names no record")
                continue
            actor = doc.footers.get(op.tag) if op.tag else None
            if op.op == "retire":
                actor_who = actor.who if actor else "agent"
                if old.source == "decided" and actor_who != "human":
                    err(op.line, "E12", "a decided record is retired by a person")
            if op.op == "attest":
                f = footer_for(op.tag, op.line, "attest")
                if f and f.how == "observed" and not cited_ok(f):
                    continue
                if f and not f.rests_on():
                    err(op.line, "E06", "attest names the new basis")
    return errors, warnings


def endpoint_ok(allowed, type_, frm_t):
    if allowed == "any":
        return True
    if allowed == "same_kind":
        return type_ == frm_t
    return type_ in allowed


def noise(slot, block, sspec, existing, repo, net, err, warn):
    text = slot.text
    if HISTORY.search(re.sub(r"`[^`]*`", "", text)):
        err(slot.line, "N01", "history belongs in the journal, not in a slot")
    if not sspec.get("date") and DATE.search(text):
        err(slot.line, "N02", "a date is a measurement's snapshot")
    if not sspec.get("units") and UNITS.search(text):
        err(slot.line, "N02", "a measurement belongs in the observation")
    if CROSSREF.search(text):
        err(slot.line, "N03", "a cross-reference is an edge line")
    spans = BACKTICK.findall(text)
    src = entity_source(repo, block.key) if "#" in block.key or "/" in block.key else ""
    if len(spans) > 2:
        err(slot.line, "N04", "more than two quoted identifiers restates the code")
    if src and len(text) >= 40 and any(text[i : i + 40] in src for i in range(0, len(text) - 39, 8)):
        err(slot.line, "N04", "forty characters of the source restated")
    if slot.name in ("claim", "text") and DIARY.search(text):
        err(slot.line, "N05", "a diary word in a claim")
    if slot.name == "claim" and RATIONALE.search(text):
        err(slot.line, "N07", "rationale leaked into the claim")
    for span in spans:
        ident = span.strip()
        if src and re.search(rf"\b{re.escape(ident)}\b", src):
            continue
        if net and (net.find(ident) or any(n["key"].endswith(ident) for n in net.nodes.values())):
            continue
        warn(slot.line, "N06", f"`{ident}` is neither in the reading nor a key")
    mine = stems(text)
    for other in existing.values():
        if other.name != slot.name and jaccard(mine, stems(other.value)) >= 0.8:
            err(slot.line, "N08", f"restates `{other.name}` ^{other.hash[:8]}")


def report(errors, warnings, limit=200):
    lines = [f"! line {l} {c} {m}" for l, c, m in sorted(errors)] + [f"? line {l} {c} {m}" for l, c, m in sorted(warnings)]
    text = "\n".join(lines)
    if len(text.encode()) > limit:
        text = text.encode()[: limit - 1].decode(errors="ignore") + "…"
    return text


def main():
    import argparse

    import net as netmod

    ap = argparse.ArgumentParser()
    ap.add_argument("--net", required=True)
    ap.add_argument("--repo", default=".")
    ap.add_argument("--task", required=True)
    ap.add_argument("--ontology", default="packs/coding/ontology.yaml")
    args = ap.parse_args()
    onto = load_ontology(os.path.join(args.repo, args.ontology))
    n = netmod.Net(args.net, args.repo)
    doc = grammar.parse(sys.stdin.read())
    errors, warnings = validate(doc, onto, n, args.task, args.repo)
    out = report(errors, warnings)
    if out:
        print(out)
    sys.exit(2 if errors else 1 if warnings else 0)


if __name__ == "__main__":
    main()
