"""Grades one scenario transcript against the fixture's ground truth.

Gates (exit 1 when missed): sufficiency — the answer states what only the full
minimal set can yield; findability — every ground-truth address appears in
relied_on. Metrics (reported, never gated): minimality precision and which gmr
verbs the agent walked with. Slow and exhaustive scans pass: the infrastructure
bar is "found", the speed layer is future work.
"""

import json
import re
import sys

from fixture import TRUTH


def last_json(text: str):
    decoder = json.JSONDecoder()
    best = None
    for match in re.finditer(r"\{", text):
        try:
            value, _ = decoder.raw_decode(text[match.start() :])
        except ValueError:
            continue
        if isinstance(value, dict) and "relied_on" in value:
            best = value
    return best


def contains(haystack: str, needle: str) -> bool:
    return needle.lower() in haystack.lower()


def grade(scenario: str, transcript: str):
    answer = last_json(transcript)
    failures = []
    report = {"scenario": scenario, "answer_found": answer is not None}
    if answer is None:
        return report, ["no trailing JSON object with relied_on in the transcript"]

    flat = json.dumps(answer)
    relied = " ".join(str(r) for r in answer.get("relied_on", []))
    truth_addresses = TRUTH["records"] + [TRUTH["said"]]

    found = [a for a in truth_addresses if contains(relied, a.split(":", 1)[1])]
    missing = [a for a in truth_addresses if a not in found]
    report["found"] = found
    report["missing"] = missing

    dragged = [d for d in TRUTH["distractors"] if contains(relied, d.split(":", 1)[1])]
    cited = len(found) + len(dragged)
    report["precision"] = round(len(found) / cited, 2) if cited else 0.0
    report["distractors_cited"] = dragged
    report["verbs_used"] = answer.get("verbs_used", [])

    if scenario in ("s1", "s2"):
        if missing:
            failures.append(f"findability: relied_on misses {missing}")
        if not contains(flat, TRUTH["token"]) and not contains(flat, TRUTH["cap"]):
            failures.append(
                "sufficiency: neither the vendor nor the contract cap reached the answer, "
                "so rates-floor.md was not actually consumed"
            )
        if not contains(flat, TRUTH["new_total"]):
            failures.append("sufficiency: the recomputed total is absent")
        if not contains(flat, "cart-total-baseline"):
            failures.append(
                "sufficiency: the recorded conclusion was not named, so the journal-only "
                "member of the set was not reached"
            )

    if scenario == "s3":
        if not contains(flat, "cart-total-baseline"):
            failures.append("the conclusion under question was not named")
        verdicts = ("moved", "drift", "no longer", "broken", "due", "not reliable", "unreliable")
        if not any(contains(flat, v) for v in verdicts):
            failures.append(
                "the agent did not report the ground as moved; a stale conclusion was "
                "relied on silently"
            )

    return report, failures


def main():
    scenario = sys.argv[1]
    transcript = open(sys.argv[2]).read()
    report, failures = grade(scenario, transcript)
    report["failures"] = failures
    print(json.dumps(report, indent=2))
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
