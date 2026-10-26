use super::NormalizerOption;
use crate::{SeparatorKind, Token, TokenKind};
use std::borrow::Cow;
use unicode_normalization::UnicodeNormalization;
fn mark(c: char) -> bool {
    matches!(c, '\u{610}'..='\u{61a}' | '\u{64b}'..='\u{65f}' | '\u{670}' | '\u{6d6}'..='\u{6ed}' | '\u{8d3}'..='\u{8ff}')
}
fn invisible(c: char) -> bool {
    matches!(
        c,
        '\u{a0}' | '\u{ad}' | '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}'
    )
}
fn fold(c: char) -> char {
    match c {
        'ك' => 'ک',
        'ي' | 'ى' | 'ٮ' | 'ئ' => 'ی',
        'ه' | 'ھ' | 'ە' | 'ۀ' => 'ہ',
        'ة' => 'ۃ',
        'ٱ' | 'أ' | 'إ' | 'آ' | 'ٲ' | 'ٳ' => 'ا',
        'ؤ' => 'و',
        _ => c,
    }
}
fn letter(c: char) -> bool {
    c.is_alphabetic() && !mark(c) && c != 'ـ'
}
fn flag(c: char) -> u16 {
    match c {
        '\u{64b}' => 1,
        '\u{65e}' => 2,
        '\u{8f0}' => 4,
        '\u{64e}' => 8,
        '\u{6e2}' => 16,
        '\u{654}' => 32,
        '\u{655}' => 64,
        '\u{650}' => 128,
        '\u{64d}' => 256,
        '\u{64f}' => 512,
        '\u{64c}' => 1024,
        _ => 0,
    }
}
fn has(mask: u16, c: char) -> bool {
    mask & flag(c) != 0
}
#[derive(Clone, Copy, Default)]
struct Context {
    previous: Option<usize>,
    next: Option<usize>,
    before: u16,
    after: u16,
}
fn contexts(units: &[(usize, char)]) -> Vec<Context> {
    let mut result = vec![Context::default(); units.len()];
    let (mut previous, mut mask) = (None, 0);
    for (i, (_, c)) in units.iter().enumerate() {
        result[i].previous = previous;
        result[i].before = mask;
        if letter(*c) {
            previous = Some(i);
            mask = 0;
        } else {
            mask |= flag(*c);
        }
    }
    let (mut next, mut mask) = (None, 0);
    for (i, (_, c)) in units.iter().enumerate().rev() {
        result[i].next = next;
        result[i].after = mask;
        if letter(*c) {
            next = Some(i);
            mask = 0;
        } else {
            mask |= flag(*c);
        }
    }
    result
}
/// Contextual rules run before any generic Arabic or diacritic normalisation.
pub(super) fn normalize<'a>(mut token: Token<'a>, options: &NormalizerOption) -> Token<'a> {
    let original: Vec<char> = token.lemma.chars().collect();
    if token.lemma.starts_with('﴿') {
        token.kind = TokenKind::Separator(SeparatorKind::Soft);
        return token;
    }
    // Keep source ownership through decomposition so all output bytes can be
    // mapped to original Unicode scalars, including expansions and deletions.
    let units: Vec<(usize, char)> = original
        .iter()
        .enumerate()
        .flat_map(|(i, c)| c.to_string().nfd().map(move |c| (i, c)).collect::<Vec<_>>())
        .collect();
    let plain: String =
        units.iter().map(|(_, c)| *c).filter(|c| letter(*c) && *c != 'ء').map(fold).collect();
    let allah =
        matches!(plain.as_str(), "اللہ" | "للہ" | "واللہ" | "باللہ" | "فاللہ" | "وللہ" | "فللہ");
    let mut pieces = vec![String::new(); original.len()];
    let contexts = contexts(&units);
    let mut skip = vec![false; units.len()];
    for (n, &(owner, c)) in units.iter().enumerate() {
        if skip[n] {
            continue;
        }
        let next = contexts[n].next;
        let prev = contexts[n].previous;
        let after = contexts[n].after;
        let before = contexts[n].before;
        if c == '\u{670}' {
            if allah {
                continue;
            }
            if let Some(i) = prev {
                let (prev_owner, prev) = units[i];
                if matches!(prev, 'و' | 'ي' | 'ى' | 'ی' | 'ٮ') {
                    pieces[prev_owner].clear();
                    pieces[owner].push(if prev == 'و' || next.is_some() { 'ا' } else { 'ی' });
                    continue;
                }
            }
            // Manzoor may put the dagger before its silent carrier instead.
            if let Some(i) = next {
                let carrier = units[i].1;
                if matches!(carrier, 'و' | 'ی' | 'ٮ' | 'ى') {
                    skip[i] = true;
                    let final_carrier = contexts[i].next.is_none();
                    pieces[owner].push(if carrier != 'و' && final_carrier { 'ی' } else { 'ا' });
                    continue;
                }
            }
            pieces[owner].push('ا');
            continue;
        }
        if matches!(c, '\u{6e7}' | '\u{6e6}' | '\u{656}') {
            // U+0656 is ambiguous across writing systems; preserve it because the
            // writing system cannot be established from a locale alone.
            if c == '\u{656}' {
                pieces[owner].push(c);
            } else if next.is_some() || (c == '\u{6e6}' && has(after, '\u{64e}')) {
                pieces[owner].push('ی');
            }
            continue;
        }
        if c == 'ا'
            && (has(before, '\u{64b}')
                || has(after, '\u{64b}')
                || has(before, '\u{65e}')
                || has(after, '\u{65e}')
                || has(before, '\u{8f0}')
                || has(after, '\u{8f0}')
                || (has(before, '\u{64e}') && has(before, '\u{6e2}')))
        {
            continue;
        }
        if c == 'ـ' {
            let hamza = has(after, '\u{654}') || has(after, '\u{655}');
            if hamza && next.is_some_and(|i| units[i].1 == 'ا') {
                continue;
            }
            if hamza && (has(after, '\u{650}') || has(after, '\u{64d}')) {
                pieces[owner].push('ی');
            } else if hamza && (has(after, '\u{64f}') || has(after, '\u{64c}')) {
                pieces[owner].push('و');
            } else if hamza && has(after, '\u{64e}') {
                pieces[owner].push('ا');
            }
            continue;
        }
        if mark(c)
            || invisible(c)
            || c.is_numeric()
            || matches!(c, 'ء' | '\u{674}' | '\u{603}' | '\u{66d}' | '\u{66a}')
        {
            continue;
        }
        if letter(c) {
            pieces[owner].push(fold(c));
        }
    }
    let mut previous = None;
    for (i, piece) in pieces.iter_mut().enumerate() {
        let mut clean = String::new();
        for c in piece.chars() {
            if !(c == 'ا' && previous == Some('ا')
                || c == 'ی' && previous == Some('ی') && original[i] == 'ئ')
            {
                clean.push(c);
            }
            previous = Some(c);
        }
        *piece = clean;
    }
    let lemma: String = pieces.iter().map(String::as_str).collect();
    token.kind = if lemma.chars().any(letter) {
        if options.classifier.stop_words.as_ref().is_some_and(|s| s.contains(&lemma)) {
            TokenKind::StopWord
        } else {
            TokenKind::Word
        }
    } else {
        TokenKind::Separator(SeparatorKind::Soft)
    };
    if options.create_char_map {
        let mut map: Vec<(u8, u8)> = original
            .iter()
            .zip(&pieces)
            .map(|(c, s)| (c.len_utf8() as u8, s.len() as u8))
            .collect();
        // Associate removed trailing marks with the preceding output character
        // so highlighting includes the marks on the final selected letter.
        for i in 1..map.len() {
            if map[i].1 == 0 {
                map[i].1 = map[i - 1].1;
                map[i - 1].1 = 0;
            }
        }
        token.char_map = Some(map);
    }
    token.lemma = Cow::Owned(lemma);
    token
}
