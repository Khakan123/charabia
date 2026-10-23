# Urdu dictionaries

Join-word data and FST for the Urdu segmenter, adapted from [urduhack](https://github.com/urduhack/urduhack) `WORDS_SPACE` (MIT License). See [NOTICE](NOTICE).

## Regenerate

From the repository root:

```bash
python charabia/dictionaries/bin/extract_urdu_joins.py
cargo run --bin build_urdu_fst -p charabia --no-default-features --features urdu-segmentation
python charabia/dictionaries/bin/generate_urdu_char_map.py
```

## Files

| File | Purpose |
|------|---------|
| `joins.tsv` | Joined form → split parts (`part1\|part2`) |
| `../fst/urdu/joins.fst` | FST built from `joins.tsv` keys |
| `NOTICE` | MIT attribution |
