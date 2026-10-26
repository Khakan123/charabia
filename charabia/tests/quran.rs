use charabia::{Language, TokenizerBuilder};
fn words(text: &str) -> Vec<String> {
    let locale = Language::from_code("qur").expect("qur locale");
    TokenizerBuilder::default()
        .build()
        .tokenize_with_allow_list(text, Some(&[locale]))
        .filter(|t| t.is_word())
        .map(|t| t.lemma.into_owned())
        .collect()
}
#[test]
fn equivalent_spellings() {
    for (a, b) in [
        ("كتاب", "کتاب"),
        ("هُدَى", "ہدی"),
        ("رَحْمَة", "رحمۃ"),
        ("اللّٰہ", "ٱللَّه"),
        ("لِلّٰہ", "لِلَّه"),
        ("الصَّلٰوۃ", "ٱلصَّلَوٰة"),
        ("عَلٰی", "عَلَىٰ"),
        ("شَيْئًا", "شیئ"),
        ("شَيۡـًٔا", "شیئ"),
        ("إِبۡرَٰهِـۧمَ", "ابراہیم"),
        ("بِهِۦ", "بہ"),
        ("وَلِيِّ\u{a0}يَ", "ولیی"),
        ("التَّوۡرٰٮۃ", "التوراۃ"),
        ("يَسۡـَٔلُ", "یسال"),
    ] {
        assert_eq!(words(a), words(b), "{a} != {b}");
    }
}
#[test]
fn boundaries_and_distinctions() {
    assert_eq!(words("الرحمن"), vec!["الرحمن"]);
    assert_ne!(words("کتاب"), words("کتب"));
    assert_ne!(words("رحمۃ"), words("رحمہ"));
    assert_eq!(words("یا قوم"), vec!["یا", "قوم"]);
    assert_eq!(words("کتاب ﴿۱۲﴾ 123 ۞ ۩"), vec!["کتاب"]);
    assert_eq!(
        words("کتاب\u{202f}کتب\u{2009}رحمۃ\u{3000}رحمہ"),
        vec!["کتاب", "کتب", "رحمۃ", "رحمہ"]
    );
    for c in ['\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}', '\u{feff}', '\u{ad}'] {
        assert_eq!(words(&format!("ک{c}تاب")), vec!["کتاب"]);
    }
}
#[test]
fn offsets() {
    let original = "وَلِيِّ\u{a0}يَ کتاب";
    let locales = [Language::from_code("qur").unwrap()];
    let mut builder = TokenizerBuilder::default();
    builder.create_char_map(true);
    let tokenizer = builder.build();
    let tokens: Vec<_> = tokenizer.tokenize_with_allow_list(original, Some(&locales)).collect();
    assert_eq!(
        tokens.iter().map(|t| &original[t.byte_start..t.byte_end]).collect::<String>(),
        original
    );
    for t in tokens.iter().filter(|t| t.is_word()) {
        let map = t.char_map.as_ref().unwrap();
        assert_eq!(map.len(), t.original_char_count());
        assert_eq!(map.iter().map(|p| p.0 as usize).sum::<usize>(), t.original_byte_len());
        assert_eq!(map.iter().map(|p| p.1 as usize).sum::<usize>(), t.byte_len());
        assert_eq!(t.original_lengths(t.byte_len()).1, t.original_byte_len());
        for (i, c) in t.lemma.char_indices() {
            let (_, end) = t.original_lengths(i + c.len_utf8());
            assert!(original.is_char_boundary(t.byte_start + end));
        }
    }
}

#[test]
fn explicit_locale_only() {
    use charabia::StrDetection;
    let q = Language::from_code("qur").unwrap();
    assert_eq!(q.code(), "qur");
    assert_eq!(serde_json::from_str::<Language>(&serde_json::to_string(&q).unwrap()).unwrap(), q);
    assert_eq!(serde_json::to_string(&Language::Ara).unwrap(), "\"Ara\"");
    let mut auto = StrDetection::new("الحمد لله رب العالمين", None);
    assert_ne!(auto.language(), Some(q));
    let mixed = [q, Language::Ara];
    let mut detection = StrDetection::new("الحمد لله رب العالمين", Some(&mixed));
    assert_eq!(detection.language(), Some(Language::Ara));
    let locales = [q];
    assert_eq!(StrDetection::new("الله", Some(&locales)).language(), Some(q));
}
#[test]
fn ambiguous_marks_and_normalization_idempotence() {
    assert_eq!(words("بِهٖ"), vec!["بہ\u{656}"]);
    for input in ["الصَّلٰوۃ", "إِبۡرَٰهِـۧمَ", "وَلِيِّ\u{a0}يَ", "رحمۃ", "الرحمن"]
    {
        let once = words(input);
        assert_eq!(words(&once.join(" ")), once);
    }
}

#[test]
fn full_span_and_idempotence_for_deleted_letters_and_ambiguous_marks() {
    let locales = [Language::from_code("qur").unwrap()];
    let mut builder = TokenizerBuilder::default();
    builder.create_char_map(true);
    let tokenizer = builder.build();
    for input in ["شَيْئًا", "بِهٖ", "رحمۃ\u{200d}", "اا", "ک\u{656}تاب"] {
        let token = tokenizer
            .tokenize_with_allow_list(input, Some(&locales))
            .find(|t| t.is_word())
            .unwrap();
        assert_eq!(token.original_lengths(token.byte_len()).1, input.len(), "{input}");
        let once = words(input);
        assert_eq!(words(&once.join(" ")), once, "{input}");
    }
}

#[test]
fn unicode_source_maps_never_split_characters() {
    fn property(input: String) -> bool {
        let locales = [Language::from_code("qur").unwrap()];
        let mut builder = TokenizerBuilder::default();
        builder.create_char_map(true);
        let tokenizer = builder.build();
        let tokens: Vec<_> = tokenizer.tokenize_with_allow_list(&input, Some(&locales)).collect();
        if tokens.iter().map(|t| &input[t.byte_start..t.byte_end]).collect::<String>() != input {
            return false;
        }
        tokens.iter().filter(|t| t.is_word()).all(|t| {
            let map = t.char_map.as_ref().unwrap();
            map.len() == t.original_char_count()
                && map.iter().map(|p| p.1 as usize).sum::<usize>() == t.byte_len()
                && t.lemma.char_indices().all(|(i, c)| {
                    input.is_char_boundary(t.byte_start + t.original_lengths(i + c.len_utf8()).1)
                })
        })
    }
    quickcheck::QuickCheck::new().tests(250).quickcheck(property as fn(String) -> bool);
}
#[test]
fn long_mark_run() {
    let input = format!("ک{}تاب", "\u{64e}".repeat(100_000));
    assert_eq!(words(&input), vec!["کتاب"]);
}
