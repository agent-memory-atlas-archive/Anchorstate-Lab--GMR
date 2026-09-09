"""The line grammar agents read and write, design.md §9.1.

    @Type key [=addr] ~s           node
    slot: text ~s [^hash]          slot value; ^hash names the record it replaces
    >rel Type:key ~s [^hash]       edge out; <rel is an edge in
      reason: text                 an indented line under an edge is that edge's reason
    retire ^hash                   the record leaves delivery
    attest ^hash ~s                the record keeps its value on a fresh basis
    ~s who how basis…              one footer per source tag

Basis, by how: observed -> rests_on list `key@addr[path,path]`; decided -> `name date`;
reviewed -> `commit` or `git`; user-said, published -> free. Any footer may end with
`rests_on key@addr[paths] …`. Addresses are 8 hex on the wire and 64 hex in a container.
"""

import re
from dataclasses import dataclass, field

TAG = re.compile(r"^~([a-z])$")
NODE = re.compile(r"^@(\w+)\s+(.+?)(?:\s+=([0-9a-f]{8,64}))?(?:\s+~([a-z]))?$")
SLOT = re.compile(r"^(!)?([a-z_]+):\s+(.*?)(?:\s+~([a-z]))?(?:\s+\^([0-9a-f]{8,64}))?$")
EDGE = re.compile(r"^([<>])(\w+)\s+(\w+):(.+?)(?:\s+~([a-z]))?(?:\s+\^([0-9a-f]{8,64}))?$")
REASON = re.compile(r"^\s+reason:\s+(.+)$")
RETIRE = re.compile(r"^retire\s+\^([0-9a-f]{8,64})$")
ATTEST = re.compile(r"^attest\s+\^([0-9a-f]{8,64})\s+~([a-z])$")
FOOTER = re.compile(r"^~([a-z])\s+(agent|human|probe)\s+(\S+)\s+(\S+)(?:\s+(.*))?$")
BASIS = re.compile(r"^(\S+?)@([0-9a-f]{8,64})(?:\[([^\]]*)\])?$")


@dataclass
class Slot:
    name: str
    text: str
    tag: str | None
    replaces: str | None
    stale: bool = False
    line: int = 0


@dataclass
class Edge:
    direction: str
    rel: str
    type: str
    key: str
    tag: str | None
    replaces: str | None
    reason: str | None = None
    line: int = 0


@dataclass
class Op:
    op: str
    hash: str
    tag: str | None
    line: int


@dataclass
class Block:
    type: str
    key: str
    addr: str | None
    tag: str | None
    slots: list = field(default_factory=list)
    edges: list = field(default_factory=list)
    ops: list = field(default_factory=list)
    line: int = 0

    @property
    def id(self):
        return f"{self.type}:{self.key}"


@dataclass
class Footer:
    tag: str
    who: str
    name: str
    how: str
    rest: str
    line: int = 0

    def rests_on(self):
        rest = self.rest
        if self.how != "observed":
            if " rests_on " in f" {rest}":
                rest = rest.split("rests_on", 1)[1]
            else:
                return []
        out = []
        for token in rest.split():
            m = BASIS.match(token)
            if m:
                paths = [p for p in (m.group(3) or "").split(",") if p]
                out.append({"key": m.group(1), "addr": m.group(2), "paths": paths})
        return out

    def basis(self):
        if self.how == "observed":
            return ""
        return self.rest.split(" rests_on ", 1)[0].strip() if " rests_on " in f" {self.rest} " else self.rest


@dataclass
class Document:
    blocks: list
    footers: dict
    problems: list


def parse(text: str) -> Document:
    blocks, footers, problems = [], {}, []
    current = None
    last_edge = None
    for n, raw in enumerate(text.splitlines(), 1):
        line = raw.rstrip()
        if not line.strip() or line.startswith("#"):
            continue
        m = REASON.match(line)
        if m and last_edge is not None:
            last_edge.reason = m.group(1).strip()
            continue
        if line.startswith((" ", "\t")):
            continue
        last_edge = None
        m = FOOTER.match(line)
        if m:
            footers[m.group(1)] = Footer(m.group(1), m.group(2), m.group(3), m.group(4), (m.group(5) or "").strip(), n)
            continue
        m = NODE.match(line)
        if m:
            current = Block(m.group(1), m.group(2).strip(), m.group(3), m.group(4), line=n)
            blocks.append(current)
            continue
        if current is None:
            problems.append((n, "E00", "a line before any @Type key"))
            continue
        m = EDGE.match(line)
        if m:
            last_edge = Edge(m.group(1), m.group(2), m.group(3), m.group(4).strip(), m.group(5), m.group(6), line=n)
            current.edges.append(last_edge)
            continue
        m = RETIRE.match(line)
        if m:
            current.ops.append(Op("retire", m.group(1), None, n))
            continue
        m = ATTEST.match(line)
        if m:
            current.ops.append(Op("attest", m.group(1), m.group(2), n))
            continue
        m = SLOT.match(line)
        if m:
            current.slots.append(Slot(m.group(2), m.group(3).strip(), m.group(4), m.group(5), bool(m.group(1)), n))
            continue
        problems.append((n, "E00", "not a node, slot, edge, footer or op line"))
    return Document(blocks, footers, problems)


def render_node(type_, key, addr=None, tag=None):
    out = f"@{type_} {key}"
    if addr:
        out += f" ={addr[:8]}"
    if tag:
        out += f" ~{tag}"
    return out


def render_slot(name, text, tag=None, hash_=None, stale=False):
    out = f"{'!' if stale else ''}{name}: {text}"
    if tag:
        out += f" ~{tag}"
    if hash_:
        out += f" ^{hash_[:8]}"
    return out


def render_edge(direction, rel, type_, key, tag=None, hash_=None, reason=None):
    out = f"{direction}{rel} {type_}:{key}"
    if tag:
        out += f" ~{tag}"
    if hash_:
        out += f" ^{hash_[:8]}"
    if reason:
        out += f"\n  reason: {reason}"
    return out


def render_footer(tag, who, name, how, basis="", rests_on=(), stale=None, short=True):
    parts = [f"~{tag}", who, name, how]
    if basis:
        parts.append(basis)
    cited = " ".join(
        f"{r['key']}@{r['addr'][:8] if short else r['addr']}" + (f"[{','.join(r['paths'])}]" if r.get("paths") else "")
        for r in rests_on
    )
    if cited:
        if how != "observed":
            parts.append("rests_on")
        parts.append(cited)
    if stale:
        parts.append(f"!stale {stale}")
    return " ".join(parts)
