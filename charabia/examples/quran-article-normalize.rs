use charabia::quran_articles::{QuranArticleForm, QuranArticleForms};
use charabia::{Language, TokenizerBuilder};
use std::io::{self, Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    if std::env::args().any(|arg| arg == "--words") {
        let words: Vec<String> = serde_json::from_str(&input)?;
        let mut builder = TokenizerBuilder::default();
        let tokenizer = builder.build();
        let mut normalized = Vec::new();
        for word in words {
            let mut tokens = tokenizer.tokenize_with_allow_list(&word, Some(&[Language::Qur]));
            let token = tokens.next().ok_or_else(|| io::Error::other("empty word"))?;
            if !token.is_word()
                || token.byte_start != 0
                || token.byte_end != word.len()
                || tokens.next().is_some()
            {
                return Err(io::Error::other("expected exactly one complete Qur’an word").into());
            }
            normalized.push(token.lemma.into_owned());
        }
        println!("{}", serde_json::to_string(&normalized)?);
    } else {
        let forms: Vec<QuranArticleForm> = serde_json::from_str(&input)?;
        let forms = QuranArticleForms::new(forms).map_err(io::Error::other)?;
        println!("{}", serde_json::to_string(forms.forms())?);
    }
    Ok(())
}
