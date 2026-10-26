/// Segments original slices. Joiners are removed only after offsets are recorded.
pub(super) struct QuranSegments<'a> {
    remaining: &'a str,
}
impl<'a> QuranSegments<'a> {
    pub(super) fn new(text: &'a str) -> Self {
        Self { remaining: text }
    }
}
fn word(c: char) -> bool {
    c.is_alphanumeric()
        || matches!(c,
        '\u{a0}' | '\u{ad}' | '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}' |
        '\u{610}'..='\u{61a}' | '\u{64b}'..='\u{65f}' | '\u{670}' | '\u{6d6}'..='\u{6ed}' | '\u{8d3}'..='\u{8ff}')
}
impl<'a> Iterator for QuranSegments<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let first = self.remaining.chars().next()?;
        let end = if first == '﴿' {
            self.remaining.find('﴾').map(|i| i + '﴾'.len_utf8()).unwrap_or(first.len_utf8())
        } else {
            let is_word = word(first);
            self.remaining
                .char_indices()
                .skip(1)
                .find(|(_, c)| *c == '﴿' || word(*c) != is_word)
                .map(|(i, _)| i)
                .unwrap_or(self.remaining.len())
        };
        let (head, tail) = self.remaining.split_at(end);
        self.remaining = tail;
        Some(head)
    }
}
