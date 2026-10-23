//! Urdu specialized [`Normalizer`].

use std::borrow::Cow;

use super::{Normalizer, NormalizerOption};
use crate::detection::Language;
use crate::{Script, Token};

#[path = "urdu_char_map.rs"]
mod urdu_char_map;

use urdu_char_map::normalize_urdu_char;

/// Combined character sequences from urduhack `COMBINE_URDU_CHARACTERS`.
const COMBINE_URDU_CHARACTERS: &[(&str, &str)] =
    &[("آ", "آ"), ("أ", "أ"), ("ۓ", "ۓ"), ("ﻻ", "لا"), ("ﻼ", "لا")];

/// Urdu specialized [`Normalizer`].
pub struct UrduNormalizer;

impl Normalizer for UrduNormalizer {
    fn normalize<'o>(&self, mut token: Token<'o>, options: &NormalizerOption) -> Token<'o> {
        if !self.should_normalize(&token) {
            return token;
        }

        let mut lemma = token.lemma.into_owned();
        for (from, to) in COMBINE_URDU_CHARACTERS {
            lemma = lemma.replace(from, to);
        }

        if options.create_char_map {
            let mut char_map = Vec::new();
            let mut normalized = String::new();
            for c in lemma.chars() {
                let char_str = c.to_string();
                let mapped: String = match map_urdu_char(c) {
                    None => char_str.clone(),
                    Some(None) => String::new(),
                    Some(Some(mapped)) => mapped.to_string(),
                };
                char_map.push((char_str.len() as u8, mapped.len() as u8));
                normalized.push_str(&mapped);
            }
            token.char_map = Some(char_map);
            token.lemma = Cow::Owned(normalized);
            return token;
        }

        let normalized: String = lemma
            .chars()
            .filter_map(|c| match map_urdu_char(c) {
                Some(Some(mapped)) => Some(mapped),
                Some(None) => None,
                None => Some(c),
            })
            .collect();

        token.lemma = Cow::Owned(normalized);
        token
    }

    fn should_normalize(&self, token: &Token) -> bool {
        token.script == Script::Arabic
            && token.language == Some(Language::Urd)
            && (token.lemma.chars().any(|c| map_urdu_char(c).is_some())
                || COMBINE_URDU_CHARACTERS.iter().any(|(from, _)| token.lemma.contains(from)))
    }
}

fn map_urdu_char(c: char) -> Option<Option<char>> {
    if let Some(mapped) = normalize_urdu_char(c) {
        return Some(mapped);
    }

    if ('۰'..='۹').contains(&c) {
        let digit = u32::from(c) - u32::from('۰') + u32::from('0');
        return Some(Some(char::from_u32(digit).unwrap()));
    }

    if c == '\u{200C}' {
        return Some(None);
    }

    None
}

#[cfg(test)]
mod test {
    use std::borrow::Cow::Owned;

    use crate::detection::Language;
    use crate::normalizer::arabic::ArabicNormalizer;
    use crate::normalizer::{CharNormalizer, Normalizer, NormalizerOption, UrduNormalizer};
    use crate::{Script, Token};

    fn urdu_token(lemma: &str) -> Token<'static> {
        let lemma = lemma.to_string();
        let char_end = lemma.chars().count();
        let byte_end = lemma.len();
        Token {
            lemma: Owned(lemma),
            char_end,
            byte_end,
            script: Script::Arabic,
            language: Some(Language::Urd),
            ..Default::default()
        }
    }

    #[test]
    fn urdu_normalizer_lemma_examples() {
        let options = NormalizerOption {
            create_char_map: false,
            lossy: true,
            classifier: crate::normalizer::ClassifierOption { stop_words: None, separators: None },
        };

        let cases = [
            ("مجھ کو جو توڑا ﮔیا تھا", "مجھ کو جو توڑا گیا تھا"),
            ("۱۲۳", "123"),
            ("جرأت", "جرأت"),
            ("پاکستان", "پاکستان"),
        ];

        for (input, expected) in cases {
            let normalized = UrduNormalizer.normalize(urdu_token(input), &options);
            assert_eq!(normalized.lemma(), expected, "input: {input}");
        }
    }

    #[test]
    fn arabic_normalizer_skips_urdu() {
        let token = urdu_token("ى");
        assert!(!CharNormalizer::should_normalize(&ArabicNormalizer, &token));
    }
}
