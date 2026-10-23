//! Urdu specialized [`Segmenter`] for joined words without spaces.
//!
//! Uses a join-word FST and split map adapted from urduhack `WORDS_SPACE` (MIT).

use std::collections::HashMap;
use std::sync::LazyLock;

use fst::raw::Fst;

use crate::segmenter::utils::{find_longest_prefix, floor_char_boundary};
use crate::segmenter::Segmenter;

static JOIN_FST: LazyLock<Fst<&'static [u8]>> = LazyLock::new(|| {
    Fst::new(&include_bytes!("../../dictionaries/fst/urdu/joins.fst")[..]).unwrap()
});

static SPLIT_MAP: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let data = include_str!("../../dictionaries/txt/urdu/joins.tsv");
    let mut map = HashMap::new();
    for line in data.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, parts)) = line.split_once('\t') else {
            continue;
        };
        let parts: Vec<String> = parts.split('|').map(str::to_string).collect();
        debug_assert_eq!(parts.concat(), key);
        map.insert(key.to_string(), parts);
    }
    map
});

/// Urdu specialized [`Segmenter`].
///
/// Splits joined Urdu words using a dictionary encoded as an FST.
/// Source: urduhack `WORDS_SPACE` (MIT License).
pub struct UrduSegmenter;

impl Segmenter for UrduSegmenter {
    fn segment_str<'o>(&self, to_segment: &'o str) -> Box<dyn Iterator<Item = &'o str> + 'o> {
        Box::new(UrduSegmentIter::new(to_segment))
    }
}

struct UrduSegmentIter<'o> {
    text: &'o str,
    offset: usize,
    buffer_start: Option<usize>,
    pending: std::vec::IntoIter<&'o str>,
}

impl<'o> UrduSegmentIter<'o> {
    fn new(text: &'o str) -> Self {
        Self { text, offset: 0, buffer_start: None, pending: Vec::new().into_iter() }
    }

    fn tail(&self) -> Option<&'o str> {
        self.text.get(self.offset..).filter(|s| !s.is_empty())
    }

    fn flush_buffer(&mut self) -> Option<&'o str> {
        let start = self.buffer_start.take()?;
        let segment = self.text.get(start..self.offset)?;
        (!segment.is_empty()).then_some(segment)
    }

    fn buffer_next_char(&mut self) {
        let _start = self.buffer_start.get_or_insert(self.offset);
        if let Some(ch) = self.text[self.offset..].chars().next() {
            self.offset += ch.len_utf8();
        }
    }

    fn try_split_match(&mut self, tail: &'o str, key_len: usize) -> Option<Vec<&'o str>> {
        let key_len = floor_char_boundary(tail, key_len);
        let key = tail.get(..key_len)?;
        let parts = SPLIT_MAP.get(key)?;
        let mut segments = Vec::with_capacity(parts.len());
        let mut part_offset = 0;
        for part in parts {
            let end = part_offset + part.len();
            let segment = tail.get(part_offset..end)?;
            debug_assert_eq!(segment, part.as_str());
            segments.push(segment);
            part_offset = end;
        }
        debug_assert_eq!(part_offset, key_len);
        self.offset += key_len;
        Some(segments)
    }
}

impl<'o> Iterator for UrduSegmentIter<'o> {
    type Item = &'o str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(segment) = self.pending.next() {
            return Some(segment);
        }

        loop {
            if self.tail().is_none() {
                return self.flush_buffer();
            }

            let tail = self.tail()?;

            if let Some((_, key_len)) = find_longest_prefix(&JOIN_FST, tail.as_bytes()) {
                if let Some(segments) = self.try_split_match(tail, key_len) {
                    if let Some(buffered) = self.flush_buffer() {
                        self.pending = segments.into_iter();
                        return Some(buffered);
                    }
                    let mut iter = segments.into_iter();
                    let first = iter.next()?;
                    self.pending = iter.collect::<Vec<_>>().into_iter();
                    return Some(first);
                }
            }

            self.buffer_next_char();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::segmenter::test::test_segmenter;

    const TEXT: &str = "عراق کردیا تھا 123 456";

    const SEGMENTED: &[&str] = &["عراق", " ", "کر", "دیا", " ", "تھا", " ", "123", " ", "456"];

    const TOKENIZED: &[&str] = &["عراق", " ", "کر", "دیا", " ", "تھا", " ", "123", " ", "456"];

    test_segmenter!(UrduSegmenter, TEXT, SEGMENTED, TOKENIZED, Script::Arabic, Language::Urd);

    #[test]
    fn joined_word_splits_lossless() {
        let segments: Vec<_> = super::UrduSegmenter.segment_str("کردیا").collect();
        assert_eq!(segments, vec!["کر", "دیا"]);
        assert_eq!(segments.concat(), "کردیا");
    }

    #[test]
    fn unjoined_word_passes_through() {
        let segments: Vec<_> = super::UrduSegmenter.segment_str("پاکستان").collect();
        assert_eq!(segments, vec!["پاکستان"]);
    }

    #[test]
    fn all_join_keys_concat_invariant() {
        for (key, parts) in super::SPLIT_MAP.iter() {
            assert_eq!(parts.concat(), *key, "split parts must rejoin to key for {key}");
        }
    }

    #[test]
    fn segmenter_lossless_on_join_keys() {
        for key in super::SPLIT_MAP.keys() {
            let segments: Vec<_> = super::UrduSegmenter.segment_str(key).collect();
            assert_eq!(segments.concat(), *key, "lossless failure for {key}");
        }
    }
}
