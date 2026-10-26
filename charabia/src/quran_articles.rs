//! Explicit, externally supplied leading-article alternatives for Qur’an queries.
//! No lexical rules are inferred and no corpus data is embedded in this module.
use crate::{Language, TokenizerBuilder};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuranArticleForm {
    pub with_article: String,
    pub without_article: String,
}

#[derive(Debug, Clone, Default)]
pub struct QuranArticleForms {
    forms: Vec<QuranArticleForm>,
    alternatives: BTreeMap<String, BTreeSet<String>>,
}

impl QuranArticleForms {
    /// Validate and normalise explicit pairs. Only one leading article is removed.
    pub fn new(forms: Vec<QuranArticleForm>) -> Result<Self, String> {
        let mut builder = TokenizerBuilder::default();
        let tokenizer = builder.build();
        let normalize = |text: &str| -> Result<String, String> {
            let text = text.trim();
            let mut words = tokenizer.tokenize_with_allow_list(text, Some(&[Language::Qur]));
            let token = words.next().ok_or("article form must not be empty")?;
            if !token.is_word()
                || token.byte_start != 0
                || token.byte_end != text.len()
                || words.next().is_some()
            {
                return Err("article form must be exactly one complete Qur’an word".into());
            }
            Ok(token.lemma.into_owned())
        };
        let mut canonical = BTreeSet::new();
        for form in forms {
            let with_article = normalize(&form.with_article)?;
            let without_article = normalize(&form.without_article)?;
            if with_article.strip_prefix("ال") != Some(without_article.as_str()) {
                return Err(
                    "article pair must differ by exactly one leading ال after strict normalisation"
                        .into(),
                );
            }
            canonical.insert(QuranArticleForm { with_article, without_article });
        }
        let mut alternatives = BTreeMap::<String, BTreeSet<String>>::new();
        for form in &canonical {
            alternatives
                .entry(form.with_article.clone())
                .or_default()
                .insert(form.without_article.clone());
            alternatives
                .entry(form.without_article.clone())
                .or_default()
                .insert(form.with_article.clone());
        }
        Ok(Self { forms: canonical.into_iter().collect(), alternatives })
    }

    pub fn forms(&self) -> &[QuranArticleForm] {
        &self.forms
    }

    /// One-hop alternatives only. The caller retains its original token and span.
    pub fn alternatives<'a>(&'a self, normalized_word: &str) -> impl Iterator<Item = &'a str> {
        self.alternatives.get(normalized_word).into_iter().flatten().map(String::as_str)
    }
}
