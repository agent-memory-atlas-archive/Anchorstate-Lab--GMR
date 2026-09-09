"""Phase 0 check: which records rest on an address that moved. Silent when nothing did.

    python3 tools/net/check.py --net NET [--count]
"""

import argparse
import sys

import net as netmod
import walk


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--net", required=True)
    ap.add_argument("--repo", default=".")
    ap.add_argument("--count", action="store_true")
    args = ap.parse_args()
    n = netmod.Net(args.net, args.repo)
    stale = []
    for r in n.records:
        marks = walk.stale_of(n, r)
        if marks:
            stale.append((r, marks))
    if args.count:
        print(len(stale))
        sys.exit(1 if stale else 0)
    for r, marks in stale:
        what = f"{r.subject}.{r.name}" if r.kind == "slot" else f"{r.subject} >{r.name} {r.value}"
        print(f"! {what} ^{r.hash[:8]} {' '.join(marks)}")
    sys.exit(1 if stale else 0)


if __name__ == "__main__":
    main()
