"""The Phase 0 net: assertion records kept in line-grammar container files.

A container is one file under the net directory. Every slot line and edge line in it
is one record; the footer its tag names gives the record its author, source class and
rests_on. Records are addressed by the sha256 of their canonical fields; the wire shows
the first eight hex characters. Writes go through `apply`, which rewrites the container
and appends the old record to history.jsonl. Addresses the runtime handed out during a
task are kept in issued/<task>.txt so a write can be refused for citing what it never saw.
"""

import hashlib
import json
import os
import subprocess
import time

import grammar


def record_hash(fields: dict) -> str:
    canonical = json.dumps(fields, sort_keys=True, ensure_ascii=False, separators=(",", ":"))
    return hashlib.sha256(canonical.encode()).hexdigest()


class Record:
    def __init__(self, kind, subject, name, value, footer, container, extra=None):
        self.kind = kind
        self.subject = subject
        self.name = name
        self.value = value
        self.who = footer.who if footer else "unknown"
        self.author = footer.name if footer else "unknown"
        self.source = footer.how if footer else "unknown"
        self.basis = footer.basis() if footer else ""
        self.rests_on = footer.rests_on() if footer else []
        self.container = container
        self.extra = extra or {}
        self.hash = record_hash(self.fields())

    def fields(self):
        return {
            "kind": self.kind,
            "subject": self.subject,
            "name": self.name,
            "value": self.value,
            "who": self.who,
            "author": self.author,
            "source": self.source,
            "basis": self.basis,
            "rests_on": self.rests_on,
            **({"reason": self.extra["reason"]} if self.extra.get("reason") else {}),
        }

    def to_json(self):
        return {"hash": self.hash, **self.fields(), "container": self.container}


class Net:
    def __init__(self, root: str, repo: str = ".", gmr_bin: str | None = None):
        self.root = root
        self.repo = repo
        self.gmr_bin = gmr_bin or os.environ.get("GMR_BIN") or os.path.join(repo, "target", "debug", "gmr")
        os.makedirs(os.path.join(root, "issued"), exist_ok=True)
        self.records = []
        self.nodes = {}
        self.docs = {}
        self._addresses = {}
        self.load()

    def load(self):
        self.records, self.nodes, self.docs = [], {}, {}
        for name in sorted(os.listdir(self.root)):
            if not name.endswith(".md"):
                continue
            path = os.path.join(self.root, name)
            with open(path, encoding="utf-8") as f:
                doc = grammar.parse(f.read())
            self.docs[name] = doc
            for block in doc.blocks:
                node = self.nodes.setdefault(block.id, {"type": block.type, "key": block.key, "containers": set()})
                node["containers"].add(name)
                footer_of = lambda tag: doc.footers.get(tag) if tag else None
                for slot in block.slots:
                    self.records.append(Record("slot", block.id, slot.name, slot.text, footer_of(slot.tag), name))
                for edge in block.edges:
                    frm, to = (block.id, f"{edge.type}:{edge.key}") if edge.direction == ">" else (f"{edge.type}:{edge.key}", block.id)
                    self.records.append(
                        Record("edge", frm, edge.rel, to, footer_of(edge.tag), name, {"reason": edge.reason})
                    )
                    self.nodes.setdefault(f"{edge.type}:{edge.key}", {"type": edge.type, "key": edge.key, "containers": set()})

    def by_hash(self, prefix: str):
        hits = [r for r in self.records if r.hash.startswith(prefix)]
        return hits[0] if len(hits) == 1 else None

    def slots_of(self, subject):
        return [r for r in self.records if r.kind == "slot" and r.subject == subject]

    def edges_out(self, subject):
        return [r for r in self.records if r.kind == "edge" and r.subject == subject]

    def edges_in(self, object_):
        return [r for r in self.records if r.kind == "edge" and r.value == object_]

    def find(self, label: str):
        if label in self.nodes:
            return [label]
        return [i for i, n in self.nodes.items() if n["key"] == label]

    def address_of(self, key: str):
        if key in self._addresses:
            return self._addresses[key]
        out = None
        try:
            r = subprocess.run(
                [self.gmr_bin, "read", key, "--json", "--lean"],
                cwd=self.repo,
                capture_output=True,
                text=True,
                timeout=120,
            )
            if r.returncode == 0:
                views = json.loads(r.stdout)
                if views:
                    v = views[0]
                    at = (v.get("facts") or {}).get("at") or {}
                    out = {
                        "addr": v.get("fact_address"),
                        "found": v.get("sighting") == "found",
                        "status": v.get("status"),
                        "kind": at.get("kind"),
                    }
        except (OSError, ValueError, subprocess.TimeoutExpired):
            out = None
        self._addresses[key] = out
        return out

    def issue(self, task: str, key: str, addr: str):
        with open(os.path.join(self.root, "issued", f"{task}.txt"), "a", encoding="utf-8") as f:
            f.write(f"{key}\t{addr}\n")

    def issued(self, task: str):
        path = os.path.join(self.root, "issued", f"{task}.txt")
        out = {}
        if os.path.exists(path):
            for line in open(path, encoding="utf-8"):
                if "\t" in line:
                    key, addr = line.rstrip("\n").split("\t", 1)
                    out.setdefault(addr, key)
        return out

    def log(self, event: dict):
        event = {"at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), **event}
        with open(os.path.join(self.root, "history.jsonl"), "a", encoding="utf-8") as f:
            f.write(json.dumps(event, ensure_ascii=False) + "\n")

    def container_for(self, subject: str, hint: str | None = None) -> str:
        node = self.nodes.get(subject)
        if node and node["containers"]:
            return sorted(node["containers"])[0]
        if hint:
            return hint
        type_, key = subject.split(":", 1)
        stem = key.split("/")[-1].replace("#", "-").replace(":", "-").replace(".", "-")
        return f"{type_.lower()}-{stem}.md"

    def append_lines(self, container: str, subject: str, lines: list, footer: str, tag: str):
        path = os.path.join(self.root, container)
        existing = open(path, encoding="utf-8").read() if os.path.exists(path) else ""
        doc = grammar.parse(existing) if existing else None
        type_, key = subject.split(":", 1)
        body = existing.rstrip("\n")
        block_present = doc is not None and any(b.id == subject for b in doc.blocks)
        footer_present = doc is not None and tag in doc.footers
        if block_present:
            if lines:
                head, sep, tail = self._split_block(body, subject)
                body = head + "\n".join([""] + lines) + sep + tail
        else:
            body = (body + "\n\n" if body else "") + "\n".join([grammar.render_node(type_, key)] + lines)
        if footer and not footer_present:
            body = body + "\n" + footer
        with open(path, "w", encoding="utf-8") as f:
            f.write(body.rstrip("\n") + "\n")

    def _split_block(self, body: str, subject: str):
        type_, key = subject.split(":", 1)
        lines = body.split("\n")
        start = next(i for i, l in enumerate(lines) if l.startswith(f"@{type_} {key}") and (l == f"@{type_} {key}" or l[len(f"@{type_} {key}")] in " ="))
        end = start + 1
        while end < len(lines) and not lines[end].startswith("@") and not lines[end].startswith("~"):
            end += 1
        while end > start + 1 and not lines[end - 1].strip():
            end -= 1
        return "\n".join(lines[:end]), "\n", "\n".join(lines[end:])

    def remove_line(self, record: Record):
        path = os.path.join(self.root, record.container)
        text = open(path, encoding="utf-8").read()
        doc = grammar.parse(text)
        lines = text.split("\n")
        target = None
        for block in doc.blocks:
            for slot in block.slots:
                if block.id == record.subject and record.kind == "slot" and slot.name == record.name and slot.text == record.value:
                    target = slot.line
            for edge in block.edges:
                frm, to = (block.id, f"{edge.type}:{edge.key}") if edge.direction == ">" else (f"{edge.type}:{edge.key}", block.id)
                if record.kind == "edge" and frm == record.subject and to == record.value and edge.rel == record.name:
                    target = edge.line
        if target is None:
            return False
        del lines[target - 1]
        if target - 1 < len(lines) and grammar.REASON.match(lines[target - 1] or ""):
            del lines[target - 1]
        with open(path, "w", encoding="utf-8") as f:
            f.write("\n".join(lines))
        return True

    def free_tag(self, container: str, taken=()) -> str:
        path = os.path.join(self.root, container)
        used = set(taken)
        if os.path.exists(path):
            used |= set(grammar.parse(open(path, encoding="utf-8").read()).footers)
        for c in "abcdefghijklmnopqrstuvwxyz":
            if c not in used:
                return c
        raise RuntimeError("no free footer tag")
