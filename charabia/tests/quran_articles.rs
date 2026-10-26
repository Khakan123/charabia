use charabia::quran_articles::{QuranArticleForm, QuranArticleForms};
use charabia::{Language, TokenizerBuilder};

fn pair(a: &str, b: &str) -> QuranArticleForm {
    QuranArticleForm { with_article: a.into(), without_article: b.into() }
}

#[test]
fn alternatives_are_bidirectional_normalized_and_not_recursive() {
    let forms = QuranArticleForms::new(vec![pair("ٱلۡكِتَٰبُ", "كِتَٰب"), pair("الکتاب", "کتاب")]).unwrap();
    assert_eq!(forms.forms().len(), 1);
    assert_eq!(forms.alternatives("الکتاب").collect::<Vec<_>>(), vec!["کتاب"]);
    assert_eq!(forms.alternatives("کتاب").collect::<Vec<_>>(), vec!["الکتاب"]);
    for untouched in ["والکتاب", "بالکتاب", "للکتاب", "الوان", "اللہ", "الذی", "نامعلوم"]
    {
        assert_eq!(forms.alternatives(untouched).count(), 0, "{untouched}");
    }
}

#[test]
fn preserves_original_token_and_spans() {
    let source = "ٱلۡكِتَٰبُ";
    let mut builder = TokenizerBuilder::default();
    builder.create_char_map(true);
    let tokenizer = builder.build();
    let token = tokenizer
        .tokenize_with_allow_list(source, Some(&[Language::Qur]))
        .find(|t| t.is_word())
        .unwrap();
    let before = token.clone();
    let forms = QuranArticleForms::new(vec![pair("الكتاب", "كتاب")]).unwrap();
    assert_eq!(forms.alternatives(token.lemma()).collect::<Vec<_>>(), vec!["کتاب"]);
    assert_eq!(token, before);
    assert_eq!(&source[token.byte_start..token.byte_end], source);
}

#[test]
fn invalid_forms_rejected_and_unknowns_not_guessed() {
    for (a, b) in [
        ("", "كتاب"),
        ("الكتاب", ""),
        ("كتاب", "كتاب"),
        ("الكتاب", "كتب"),
        ("والكتاب", "كتاب"),
        ("الكتاب", "كتاب نور"),
        ("الكتاب!", "كتاب"),
    ] {
        assert!(QuranArticleForms::new(vec![pair(a, b)]).is_err(), "{a} / {b}");
    }
    assert!(QuranArticleForms::new(vec![]).unwrap().alternatives("الکتاب").next().is_none());
}
