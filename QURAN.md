# Explicit Qur’an locale

## Easy overview: what is custom?

These changes affect matching, not the original text stored or displayed.

| Area | Custom change | What it means |
| --- | --- | --- |
| Locale | New `qur` mode in Charabia and Meilisearch | Select it in `localizedAttributes` and search `locales`. |
| Activation | Explicit selection only | Only `qur` on its own activates this pipeline. Automatic detection never chooses it; mixed locale lists use ordinary detection. |
| Engine connection | Meilisearch uses the sibling patched Charabia | Locale parsing, validation and conversions support `qur`. |
| Whole words | Preserve initial **ال** | The prefix stays part of the searchable word. |
| Manzoor joins | Join across non-breaking spaces (NBSP) | Remove the joiner during normalisation without splitting the word. Ordinary spaces remain boundaries. |
| Other spaces | Narrow, thin and ideographic spaces separate words | These do not act as Manzoor joins. |
| Invisible characters | Keep supported invisible characters until normalisation | Remove them without accidentally splitting words. |
| Annotations | Exclude bracketed ayah annotations, standalone numbers and ornaments | They do not become searchable words. |
| Keyboard variants | Fold Arabic/Urdu kaf, yeh, heh and alif variants | Equivalent supported spellings use shared forms for matching. |
| Letter distinctions | **ة → ۃ** and **ھ → ہ** | **ۃ remains different from ہ**. |
| Dagger alif | Interpret neighbouring letters | Separate handling for Allah/lillah forms, silent waw carriers, and medial versus final yeh carriers. |
| Hamza | Handle seats and hamza on tatweel contextually | Inspect surrounding marks before removing them. |
| Tanween and small letters | Handle tanween-related alif and contextual small yeh | Interpret supported contexts before generic mark cleanup. |
| Cleanup | Remove remaining supported diacritics, small waw, tatweel, ornaments and invisible characters | Also clean up duplicate alifs and the supported yeh-plus-hamza-seat combination. |
| Processing order | Bypass the generic Arabic normaliser in Qur’an mode | Later normalisation cannot undo the custom distinctions; ordinary Arabic/Urdu processing is unchanged. |
| Highlighting | Map normalised text back to original character and byte positions | Support joins, replacements, expansions and removals while retaining original text and combining marks in highlights. |
| Automated checks | Add normalisation, tokenisation, locale and Unicode-offset tests | Check supported matches and distinctions that must remain. |
| Server checks | Add a fixture corpus and HTTP test/benchmark script | Exercise indexing, search, highlighting, cropping, updates and restart persistence; not a full-corpus accuracy guarantee. |
| Ambiguous marks | Preserve U+0656; remove U+0657 without using it alone to delete alif | Source-aware interpretation remains unresolved rather than guessed. |
| Excluded: loose matching | No blanket alif removal or **ۃ → ہ** | No stemming, plural inference or joining words across ordinary spaces. |
| Excluded: later integration | No corpus dictionary, root-search mappings or ranking changes | Directus indexing, result merging and portal integration remain separate work. |

## Technical notes

Use `Language::Qur` / `Language::from_code("qur")` with a sole-locale tokenizer
allow-list. Automatic detection never selects it. Existing enum variants retain
their serialized names and order; Qur is appended. The compatibility conversion
to whatlang maps Qur to Ara, but detection explicitly excludes Qur candidates.

The Qur’an pipeline segments original slices before normalisation and preserves
character/byte spans. It uses its own ordered contextual normalisation instead
of the ordinary Arabic rules. No loose matching, morphology or corpus data is
included. U+0656 is intentionally retained because its source-specific meaning
cannot be inferred reliably from the locale alone.

Baseline: `bc63b9860434f06cf78b51bb0eab00aaefb69f15`, package 0.10.0.
Use Rust 1.98.1; run `cargo +1.98.1 test -p charabia --no-default-features`.
The sibling Meilisearch fork contains HTTP fixtures, benchmark/restart checks and
complete build notes in `documentation/quran-fork.md`.
