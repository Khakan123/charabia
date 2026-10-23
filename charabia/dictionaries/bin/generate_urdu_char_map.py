#!/usr/bin/env python3
"""Generate Rust char mapping for UrduNormalizer from urduhack."""

import ast
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CHAR_PY = ROOT / "urduhack-python" / "urduhack" / "normalization" / "character.py"
OUT = Path(__file__).resolve().parents[2] / "src" / "normalizer" / "urdu_char_map.rs"


def char_lit(c: str) -> str:
    if not c:
        return "''"
    cp = ord(c)
    if cp < 0x80 and c.isprintable() and c not in ("\\", "'"):
        return f"'{c}'"
    return f"'\\u{{{cp:04x}}}'"


def main() -> None:
    src = CHAR_PY.read_text(encoding="utf-8")
    match = re.search(
        r"_CORRECT_URDU_CHARACTERS_MAPPING.*?=\s*(\{.*?\n\s*\})", src, re.DOTALL
    )
    mapping = ast.literal_eval(match.group(1))

    lines = [
        "// Auto-generated from urduhack normalization/character.py — do not edit.",
        "",
        "/// `None` = keep, `Some(None)` = remove, `Some(Some(c))` = replace.",
        "pub(super) fn normalize_urdu_char(c: char) -> Option<Option<char>> {",
        "    match c {",
    ]

    for correct, wrongs in sorted(mapping.items(), key=lambda x: x[0]):
        if len(correct) != 1:
            continue
        for wrong in wrongs:
            if not wrong or len(wrong) != 1:
                continue
            w = char_lit(wrong)
            arm = (
                f"        {w} => Some(None),"
                if not correct
                else f"        {w} => Some(Some({char_lit(correct)})),"
            )
            if arm not in lines:
                lines.append(arm)

    lines.extend(
        [
            "        _ => None,",
            "    }",
            "}",
            "",
        ]
    )

    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {OUT}")


if __name__ == "__main__":
    main()
