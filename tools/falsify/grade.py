"""Aggregate results.jsonl into the per-arm table and the pre-registered verdict.

    python3 grade.py <results.jsonl>
"""

import json
import sys


def mean(xs):
    xs = [x for x in xs if x is not None]
    return sum(xs) / len(xs) if xs else None


def main():
    rows = [json.loads(l) for l in open(sys.argv[1], encoding="utf-8") if l.strip()]
    latest = {}
    for r in rows:
        latest[(r["arm"], int(r["task"]))] = r
    arms = sorted({a for a, _ in latest})
    print("| arm | task | outside before edit | tests | writes landed/attempted | stale | cost |")
    print("|---|---|---|---|---|---|---|")
    for arm in arms:
        for task in range(1, 11):
            r = latest.get((arm, task))
            if not r:
                continue
            m = r["meter"]
            print(
                f"| {arm} | {task} | {m.get('outside_before_edit')} | {'pass' if r.get('tests_pass') else 'fail'} | "
                f"{r.get('writes_landed', '-')}/{r.get('writes_attempted', '-')} | {r.get('stale', '-')} | "
                f"{(m.get('total_cost_usd') or 0):.2f} |"
            )
    net = [latest.get(("net", t)) for t in range(1, 11)]
    early = mean([r["meter"]["outside_before_edit"] for r in net[:5] if r])
    late = mean([r["meter"]["outside_before_edit"] for r in net[5:] if r])
    landed = sum(r.get("writes_landed", 0) for r in net if r)
    attempted = sum(r.get("writes_attempted", 0) for r in net if r)
    print()
    if early is not None and late is not None:
        ratio = late / early if early else None
        print(f"net arm outside bytes: tasks 1-5 mean {early:.0f}, tasks 6-10 mean {late:.0f}, ratio {ratio if ratio is None else round(ratio, 2)}")
    print(f"net arm writes landed: {landed}/{attempted}")
    empty = [k for k, r in latest.items() if not r["meter"].get("calls")]
    complete = all(latest.get(("net", t)) for t in range(1, 11))
    if empty:
        print(f"verdict: invalid, {len(empty)} cells have no transcript: {sorted(empty)}")
    elif complete:
        falsified = (late is None or early is None or early == 0 or late / early >= 0.7) or (attempted and landed / attempted < 0.5)
        print("verdict:", "FALSIFIED" if falsified else "survives")
    else:
        print("verdict: incomplete, net arm has not run all ten tasks")


if __name__ == "__main__":
    main()
