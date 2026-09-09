# gmr net, in three commands

This repository keeps what is known about its code as a net of one-line assertions
tied to readings of the code. Before you change a function, type or file, walk it;
when you are done, write back what a later agent should know.

    export GMR_BIN=GMRBIN
    python3 tools/net/walk.py  --net NET --task TASK <path#name>   -> the node, its slots, one hop of edges
    python3 tools/net/write.py --net NET --task TASK < doc          -> land slots, edges, Decisions; refuses noise
    python3 tools/net/check.py --net NET                             -> what rests on a reading that moved; silent when nothing

Wire grammar, the same for reading and writing:

    @Type key                 a node: Function, Type, Field, Test, Module, File, Decision, Policy, Claim, Concept
    slot: one sentence. ~s    a slot value; `!` in front in walk output means its reading moved
    >rel Type:key ~s          an edge out (`<rel` is an edge in); an indented `reason:` line belongs to the edge
    slot: text ~s ^hash       replaces the record walk showed as ^hash; without ^hash a second value is refused
    retire ^hash              the record leaves delivery
    ~s who how basis          one footer per source: `~a agent TASK observed path#name@addr`
                              or `~h human NAME decided DATE`; addr is what walk printed after `=`

What the validator enforces; a refusal names the line and the code:

- only a slot the type declares: Function has purpose, contract, fails_by; Type means, invariant;
  Decision claim, rationale, review_question, standing; Claim text. `python3 tools/net/walk.py`
  on a node lists what is there.
- `observed` must cite an address walk handed you in this task, in the footer
- one sentence per slot; no dates, no units, no "used to / once / no longer", no [[links]],
  no quoted source. A cross-reference is an edge line.
- Decision and Policy are a person's; you never write `decided`. Your own conclusion is a
  Claim: `@Claim TASK-1` with `text:` and a footer `~a agent TASK observed …`.
- calls, imports, contains, implements, reads, exposes are readings; the probe writes them, you do not.

Exit: 0 landed · 1 landed with warnings · 2 refused, nothing written.
