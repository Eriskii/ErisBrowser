//! UTF-8-safe text selection used by native chrome and basic form fields.
use std::ops::Range;

#[derive(Default)]
pub struct Selection {
    pub caret: usize,
    pub anchor: usize,
}
impl Selection {
    pub fn range(&self) -> Range<usize> {
        self.caret.min(self.anchor)..self.caret.max(self.anchor)
    }
    pub fn collapsed(&self) -> bool {
        self.caret == self.anchor
    }
    pub fn end(&mut self, text: &str) {
        self.caret = text.len();
        self.anchor = self.caret;
    }
    pub fn all(&mut self, text: &str) {
        self.anchor = 0;
        self.caret = text.len();
    }
    pub fn normalize(&mut self, text: &str) {
        self.caret = boundary(text, self.caret);
        self.anchor = boundary(text, self.anchor);
    }
    pub fn replace(&mut self, text: &mut String, insert: &str, limit: usize) -> bool {
        self.normalize(text);
        let range = self.range();
        if (text.len() - range.len())
            .checked_add(insert.len())
            .is_none_or(|length| length > limit)
        {
            return false;
        }
        let changed = &text[range.clone()] != insert;
        if changed {
            text.replace_range(range.clone(), insert);
        }
        self.caret = range.start + insert.len();
        self.anchor = self.caret;
        changed
    }
    pub fn erase(&mut self, text: &mut String, backward: bool) -> bool {
        self.normalize(text);
        if self.collapsed() {
            if backward {
                self.anchor = text[..self.caret]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
            } else {
                self.anchor = self.caret
                    + text[self.caret..]
                        .chars()
                        .next()
                        .map(char::len_utf8)
                        .unwrap_or(0);
            }
        }
        self.replace(text, "", usize::MAX)
    }
    pub fn horizontal(&mut self, text: &str, right: bool, extend: bool) {
        self.normalize(text);
        if !extend && !self.collapsed() {
            self.caret = if right {
                self.range().end
            } else {
                self.range().start
            };
        } else if right {
            self.caret += text[self.caret..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);
        } else {
            self.caret = text[..self.caret]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
        if !extend {
            self.anchor = self.caret;
        }
    }
    pub fn edge(&mut self, text: &str, end: bool, extend: bool) {
        self.caret = if end { text.len() } else { 0 };
        if !extend {
            self.anchor = self.caret;
        }
    }
}
fn boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scalar_movement_and_deletion_preserve_utf8() {
        let mut text = "aé🦀z".to_string();
        let mut selection = Selection::default();
        selection.end(&text);
        selection.horizontal(&text, false, false);
        selection.erase(&mut text, true);
        assert_eq!(text, "aéz");
        selection.erase(&mut text, false);
        assert_eq!(text, "aé");
        selection.horizontal(&text, false, false);
        assert_eq!(selection.caret, 1);
    }
    #[test]
    fn reversed_selection_replacement_and_limits() {
        let mut text = "abc日本語xyz".to_string();
        let mut selection = Selection {
            caret: 3,
            anchor: 12,
        };
        assert!(selection.replace(&mut text, "é", 16));
        assert_eq!(text, "abcéxyz");
        selection.all(&text);
        assert!(!selection.replace(&mut text, "toolong", 2));
        assert_eq!(text, "abcéxyz");
        assert!(selection.replace(&mut text, "ok", 2));
        assert_eq!(text, "ok");
    }
    #[test]
    fn shifted_movement_extends_then_collapses_selection() {
        let text = "abc";
        let mut selection = Selection::default();
        selection.horizontal(text, true, true);
        selection.horizontal(text, true, true);
        assert_eq!(selection.range(), 0..2);
        selection.horizontal(text, false, false);
        assert!(selection.collapsed());
        assert_eq!(selection.caret, 0);
        selection.edge(text, true, true);
        assert_eq!(selection.range(), 0..3);
    }
    #[test]
    fn no_op_replacements_and_boundary_deletions_report_no_change() {
        let mut text = "é🦀".to_owned();
        let mut selection = Selection::default();
        assert!(!selection.erase(&mut text, true));
        selection.end(&text);
        assert!(!selection.erase(&mut text, false));
        assert!(!selection.replace(&mut text, "", 32));
        selection.all(&text);
        assert!(!selection.replace(&mut text, "é🦀", 32));
        assert!(selection.collapsed());
        assert_eq!(selection.caret, text.len());
        assert_eq!(text, "é🦀");
        assert!(selection.erase(&mut text, true));
        assert_eq!(text, "é");
    }
}
