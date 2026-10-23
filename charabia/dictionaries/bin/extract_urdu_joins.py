#!/usr/bin/env python3
"""Extract WORDS_SPACE from urduhack into joins.tsv for charabia."""

import ast
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
WORDS_PY = ROOT / "urduhack-python" / "urduhack" / "tokenization" / "words.py"
OUT_DIR = Path(__file__).resolve().parents[1] / "txt" / "urdu"
OUT_TSV = OUT_DIR / "joins.tsv"


def load_words_space(path: Path) -> dict[str, str]:
    source = path.read_text(encoding="utf-8")
    match = re.search(r"WORDS_SPACE:\s*Dict\[str,\s*str\]\s*=\s*(\{.*?\n\s*\})", source, re.DOTALL)
    if not match:
        raise RuntimeError("Could not find WORDS_SPACE dict in words.py")
    return ast.literal_eval(match.group(1))


def main() -> int:
    if not WORDS_PY.exists():
        print(f"error: {WORDS_PY} not found", file=sys.stderr)
        return 1

    words_space = load_words_space(WORDS_PY)
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    lines = [
        "# Urdu join-word dictionary for charabia FST segmenter",
        "# Source: urduhack WORDS_SPACE (MIT License, Copyright 2018 Urdu Hack)",
        "# https://github.com/urduhack/urduhack",
        "# Format: joined_form<TAB>part1|part2|...",
        "",
    ]

    skipped = []
    valid = 0
    for key, value in sorted(words_space.items()):
        parts = value.split()
        if "".join(parts) != key:
            skipped.append(f"  {key!r} -> {value!r}")
            continue
        lines.append(f"{key}\t{'|'.join(parts)}")
        valid += 1

    OUT_TSV.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Wrote {valid} entries to {OUT_TSV}")

    if skipped:
        print(f"skipped {len(skipped)} invalid entries:", file=sys.stderr)
        for err in skipped:
            print(err, file=sys.stderr)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
