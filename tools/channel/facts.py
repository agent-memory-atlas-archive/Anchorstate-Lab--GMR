"""Pre-registered fact checklist for the currency-change task on the emergence
fixture. Written before any run; do not tune after seeing transcripts.

Shared facts (world-derivable AND present in the corpus) decide criterion 1.
F2 lives only in reporter source; F6 lives only in the gmr journal - each arm
is expected to miss the fact its channel cannot carry, and that asymmetry is a
result, not a grading error.
"""

import json
import re
import sys

SHARED = {
    "F1_consumer_parse_entry": r"parse_entry",
    "F3_dual_shape_forever": r"append.only",
    "F3b_legacy_lines_persist": r"(both|legacy|3.field|three.field|coexist|persist)",
    "F4_exact_cents_no_float": r"(f64|float|truncat|exact(ly)? to the cent|string manipulation|never.*float)",
    "F5_currency_aware_totals": r"(per.currency|group(ed)?.by.currenc|mix(ed)?.currenc|single.currency|currenc(y|ies).*(group|separate|sum|total)|total.*currenc)",
}
CODE_ONLY = {
    "F2_trailing_field_only": r"(trailing|fourth field|4th field|last field|append(ed)? (as|at|to) the end|end of (the )?line|final field)",
}
CORPUS_ONLY = {
    "F6_standing_records_due": r"(lose.? ground|loses ground|handed back|come.? due|is due|retire|reaffirm|accept --why|gmr accept)",
}
CONTAMINATION = {
    "ingest_marked_affected": r"ingest",
}


def last_json(text: str):
    decoder = json.JSONDecoder()
    best = None
    for match in re.finditer(r"\{", text):
        try:
            value, _ = decoder.raw_decode(text[match.start():])
        except ValueError:
            continue
        if isinstance(value, dict) and "relied_on" in value:
            best = value
    return best


def score(answer: dict):
    flat = json.dumps(answer).lower()
    hit = lambda rx: bool(re.search(rx, flat))
    shared = {k: hit(rx) for k, rx in SHARED.items()}
    f3a = shared.pop("F3_dual_shape_forever")
    f3b = shared.pop("F3b_legacy_lines_persist")
    shared["F3_dual_shape_forever"] = f3a and f3b
    return {
        "C_shared": round(sum(shared.values()) / len(shared), 2),
        "shared": shared,
        "F2_code_only": {k: hit(rx) for k, rx in CODE_ONLY.items()},
        "F6_corpus_only": {k: hit(rx) for k, rx in CORPUS_ONLY.items()},
        "contamination_flags": {k: hit(rx) for k, rx in CONTAMINATION.items()},
    }


if __name__ == "__main__":
    answer = last_json(open(sys.argv[1]).read())
    if answer is None:
        print(json.dumps({"error": "no final JSON with relied_on"}))
        sys.exit(1)
    print(json.dumps(score(answer), indent=2))
