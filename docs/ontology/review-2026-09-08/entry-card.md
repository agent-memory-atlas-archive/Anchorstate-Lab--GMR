---
name: gmr
description: Trigger when `.anchor/` exists. Before changing code, `gmr enter`; when done, `gmr write` what fills a slot.
---
# gmr in five verbs

    gmr locate <words|path#name>    -> ≤5 lines: Type key
    gmr enter <action> <key>        -> the subgraph an action needs (modify|extend|explain|verify|debug)
    gmr read <key>|ontology[.Type]  -> one node; or the slots and relations a Type has
    gmr write < doc                 -> land slots, edges, Decisions; refuses noise
    gmr check [key]                 -> only what went stale; silent when nothing did

Wire grammar, same for read and write:

    @Type key ~s            a node; ~s names a source defined at the end
    slot: one sentence. ~s  `!` in front: its reading moved since it was written
    >rel Type:key ~s        edge out (`<rel` edge in); indented lines = far node's slots
    ~s who how address      `agent <task> observed <addr>` | `human <name> decided <date>`

Rules the runtime enforces; a refusal names the line and the code:
- only what fills a declared slot; `gmr read ontology.<Type>` lists them
- `observed` needs an address that enter/read handed you this task
- slot text: one sentence; no dates, units, "used to/once/no longer", [[links]], quoted code. A cross-reference is an edge line.
- Decision/Policy carry claim, rationale, ask; a person decides them, never you
- your own inference is a Claim with `>rests_on <addr>`; check retires it when the address moves

Exit: 0 done · 1 stale or warned · 2 refused, nothing written
