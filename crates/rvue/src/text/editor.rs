//! Text editing state for TextInput widgets.
//!
//! Provides text editing capabilities including:
//! - Text content management
//! - Selection/cursor position tracking
//! - IME composition support
//! - Basic text editing operations

use rudo_gc::{Gc, GcCell, Trace};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

unsafe impl Trace for Selection {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn cursor(&self) -> usize {
        self.end.max(self.start)
    }

    pub fn from_cursor(cursor: usize) -> Self {
        Self { start: cursor, end: cursor }
    }

    pub fn len(&self) -> usize {
        self.end.abs_diff(self.start)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ImeComposition {
    pub text: String,
    pub cursor_start: usize,
    pub cursor_end: usize,
}

unsafe impl Trace for ImeComposition {
    fn trace(&self, _visitor: &mut impl rudo_gc::Visitor) {}
}

impl ImeComposition {
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_start = 0;
        self.cursor_end = 0;
    }
}

pub struct TextEditor {
    pub content: GcCell<String>,
    pub selection: GcCell<Selection>,
    pub composition: GcCell<ImeComposition>,
}

unsafe impl Trace for TextEditor {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.content.trace(visitor);
        self.selection.trace(visitor);
        self.composition.trace(visitor);
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEditor {
    pub fn new() -> Self {
        Self {
            content: GcCell::new(String::new()),
            selection: GcCell::new(Selection::default()),
            composition: GcCell::new(ImeComposition::default()),
        }
    }

    pub fn with_text(text: &str) -> Self {
        Self {
            content: GcCell::new(text.to_string()),
            selection: GcCell::new(Selection::default()),
            composition: GcCell::new(ImeComposition::default()),
        }
    }

    fn char_index_to_byte_index(&self, char_index: usize) -> usize {
        self.content
            .borrow()
            .char_indices()
            .nth(char_index)
            .map(|(i, _)| i)
            .unwrap_or(self.content.borrow().len())
    }

    fn char_count(&self) -> usize {
        self.content.borrow().chars().count()
    }

    pub fn content(&self) -> String {
        self.content.borrow().clone()
    }

    pub fn text_with_composition(&self) -> String {
        let content = self.content.borrow();
        let composition = self.composition.borrow();
        if composition.is_empty() {
            let result = content.clone();
            log::debug!("text_with_composition: empty, returning '{}'", result);
            result
        } else {
            let cursor_offset = content.chars().count().min(composition.cursor_start);
            let content_chars: Vec<char> = content.chars().collect();
            let left: String = content_chars[..cursor_offset].iter().collect();
            let right: String = content_chars[cursor_offset..].iter().collect();
            let full_text = format!("{}{}{}", left, composition.text, right);
            log::debug!(
                "text_with_composition: content='{}', cursor_offset={}, left='{}', preedit='{}', right='{}', result='{}'",
                content, cursor_offset, left, composition.text, right, full_text
            );
            full_text
        }
    }

    pub fn composition_cursor_offset(&self) -> usize {
        let content = self.content.borrow();
        let composition = self.composition.borrow();
        if composition.is_empty() {
            let count = content.chars().count();
            log::debug!("composition_cursor_offset: empty, returning char count={}", count);
            count
        } else {
            let content_count = content.chars().count();
            let offset = content_count.min(composition.cursor_start);
            log::debug!(
                "composition_cursor_offset: content='{}' ({} chars), cursor_start={}, returning={}",
                content,
                content_count,
                composition.cursor_start,
                offset
            );
            offset
        }
    }

    pub fn preedit_start_offset(&self) -> usize {
        let offset = self.composition_cursor_offset();
        log::debug!("preedit_start_offset: returning {}", offset);
        offset
    }

    pub fn set_content(&self, text: String) {
        *self.content.borrow_mut_gen_only() = text;
        *self.selection.borrow_mut_gen_only() = Selection::default();
    }

    pub fn selection(&self) -> Selection {
        *self.selection.borrow()
    }

    pub fn set_selection(&self, selection: Selection) {
        *self.selection.borrow_mut_gen_only() = selection;
    }

    pub fn select(&self, start: usize, end: usize) {
        let text_len = self.content.borrow().len();
        let start = start.min(text_len);
        let end = end.min(text_len);
        *self.selection.borrow_mut_gen_only() = Selection::new(start, end);
    }

    pub fn select_all(&self) {
        let len = self.content.borrow().len();
        *self.selection.borrow_mut_gen_only() = Selection::new(0, len);
    }

    pub fn collapse_selection(&self) {
        let cursor = self.selection.borrow().cursor();
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(cursor);
    }

    pub fn is_composing(&self) -> bool {
        !self.composition.borrow().is_empty()
    }

    pub fn composition(&self) -> ImeComposition {
        self.composition.borrow().clone()
    }

    pub fn set_composition(&self, text: &str, cursor_range: Option<(usize, usize)>) {
        let (start, end) = cursor_range.unwrap_or((text.len(), text.len()));
        log::debug!(
            "set_composition: text='{}', cursor_range={:?}, cursor_start={}, cursor_end={}",
            text,
            cursor_range,
            start,
            end
        );
        *self.composition.borrow_mut_gen_only() =
            ImeComposition { text: text.to_string(), cursor_start: start, cursor_end: end };
    }

    pub fn clear_composition(&self) {
        self.composition.borrow_mut_gen_only().clear();
    }

    pub fn commit_composition(&self) {
        let (cursor, new_text) = {
            let composition = self.composition.borrow();
            if composition.is_empty() {
                return;
            }
            let cursor = self.selection.borrow().cursor();
            let new_text = composition.text.clone();
            (cursor, new_text)
        };

        let new_cursor = cursor + new_text.len();

        {
            let mut content = self.content.borrow_mut_gen_only();
            let old_text = content.clone();
            let new_content = format!("{}{}{}", &old_text[..cursor], new_text, &old_text[cursor..]);
            *content = new_content;
        }

        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(new_cursor);
        self.clear_composition();
    }

    pub fn delete_selection(&self) {
        let selection = self.selection.borrow();
        if selection.is_empty() {
            return;
        }

        let start = selection.start.min(selection.end);
        let end = selection.end.max(selection.start);

        let start_byte = self.char_index_to_byte_index(start);
        let end_byte = self.char_index_to_byte_index(end);

        {
            let mut content = self.content.borrow_mut_gen_only();
            let old_text = content.clone();
            let new_content = format!("{}{}", &old_text[..start_byte], &old_text[end_byte..]);
            *content = new_content;
        }

        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(start);
    }

    pub fn insert_text(&self, text: &str) {
        self.delete_selection();

        let cursor = self.selection.borrow().cursor();
        let cursor_byte = self.char_index_to_byte_index(cursor);

        {
            let mut content = self.content.borrow_mut_gen_only();
            let old_text = content.clone();
            let new_content =
                format!("{}{}{}", &old_text[..cursor_byte], text, &old_text[cursor_byte..]);
            *content = new_content;
        }

        let new_cursor = cursor + text.chars().count();
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(new_cursor);
    }

    pub fn backspace(&self) {
        let cursor = {
            let selection = self.selection.borrow();
            if !selection.is_empty() {
                self.delete_selection();
                return;
            }
            selection.cursor()
        };

        if cursor == 0 {
            return;
        }

        let prev_char_byte = self.char_index_to_byte_index(cursor - 1);
        let cursor_byte = self.char_index_to_byte_index(cursor);

        {
            let mut content = self.content.borrow_mut_gen_only();
            let old_text = content.clone();
            let new_content =
                format!("{}{}", &old_text[..prev_char_byte], &old_text[cursor_byte..]);
            *content = new_content;
        }

        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(cursor - 1);
    }

    pub fn delete(&self) {
        let selection = self.selection.borrow();
        if !selection.is_empty() {
            self.delete_selection();
            return;
        }

        let cursor = selection.cursor();
        let char_count = self.char_count();

        if cursor >= char_count {
            return;
        }

        let cursor_byte = self.char_index_to_byte_index(cursor);
        let next_char_byte = self.char_index_to_byte_index(cursor + 1);

        {
            let mut content = self.content.borrow_mut_gen_only();
            let old_text = content.clone();
            let new_content =
                format!("{}{}", &old_text[..cursor_byte], &old_text[next_char_byte..]);
            *content = new_content;
        }
    }

    pub fn move_cursor(&self, delta: isize) {
        let selection = self.selection.borrow();
        let cursor = selection.cursor();
        let char_count = self.char_count();
        let new_cursor = (cursor as isize + delta).clamp(0, char_count as isize) as usize;
        drop(selection);

        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(new_cursor);
    }

    pub fn move_to(&self, position: usize) {
        let char_count = self.char_count();
        let position = position.min(char_count);
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(position);
    }

    pub fn move_to_start(&self) {
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(0);
    }

    pub fn move_to_end(&self) {
        let len = self.char_count();
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(len);
    }

    pub fn move_word_left(&self) {
        let cursor = self.selection.borrow().cursor();
        let content = self.content.borrow();

        let grapheme_count = content.chars().count();
        if cursor == 0 || grapheme_count == 0 {
            return;
        }

        let before_cursor: String = content.chars().take(cursor).collect();
        let grapheme_indices: Vec<usize> = before_cursor
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(before_cursor.len()))
            .collect();

        let mut new_cursor = 0usize;
        for i in (0..grapheme_indices.len() - 1).rev() {
            let grapheme_start = grapheme_indices[i];
            let grapheme_end = grapheme_indices[i + 1];
            let grapheme = &before_cursor[grapheme_start..grapheme_end];

            if grapheme.chars().next().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
                new_cursor = i;
            } else if new_cursor != 0 {
                break;
            } else {
                new_cursor = i;
            }
        }

        drop(content);
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(new_cursor);
    }

    pub fn move_word_right(&self) {
        let cursor = self.selection.borrow().cursor();
        let content = self.content.borrow();

        let grapheme_count = content.chars().count();
        if cursor >= grapheme_count {
            *self.selection.borrow_mut_gen_only() = Selection::from_cursor(grapheme_count);
            return;
        }

        let after_cursor: String = content.chars().skip(cursor).collect();
        let grapheme_indices: Vec<usize> = after_cursor
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(after_cursor.len()))
            .collect();

        let mut found_word = false;
        let mut new_cursor = cursor;

        for i in 0..grapheme_indices.len() - 1 {
            let grapheme = &after_cursor[grapheme_indices[i]..grapheme_indices[i + 1]];
            let is_word =
                grapheme.chars().next().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);

            if is_word {
                found_word = true;
            } else if found_word {
                break;
            }
            new_cursor += 1;
        }

        drop(content);
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(new_cursor);
    }

    pub fn delete_word(&self) {
        let cursor = self.selection.borrow().cursor();
        let content = self.content.borrow();

        let grapheme_count = content.chars().count();
        if cursor >= grapheme_count {
            return;
        }

        let after_cursor: String = content.chars().skip(cursor).collect();
        let grapheme_indices: Vec<usize> = after_cursor
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(after_cursor.len()))
            .collect();

        let mut word_end = cursor;
        for i in 0..grapheme_indices.len() - 1 {
            let grapheme = &after_cursor[grapheme_indices[i]..grapheme_indices[i + 1]];
            if grapheme.chars().next().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
                word_end = cursor + i + 1;
            } else if word_end > cursor {
                break;
            }
        }

        if word_end > cursor {
            let cursor_byte = self.char_index_to_byte_index(cursor);
            let word_end_byte = self.char_index_to_byte_index(word_end);

            let mut content_mut = self.content.borrow_mut_gen_only();
            let old_text = content_mut.clone();
            let new_content = format!("{}{}", &old_text[..cursor_byte], &old_text[word_end_byte..]);
            *content_mut = new_content;
            *self.selection.borrow_mut_gen_only() = Selection::from_cursor(cursor);
        }
    }

    pub fn backdelete_word(&self) {
        let selection = self.selection.borrow();
        if !selection.is_empty() {
            drop(selection);
            self.delete_selection();
            return;
        }

        let cursor = selection.cursor();
        if cursor == 0 {
            return;
        }

        let content = self.content.borrow();

        let before_cursor: String = content.chars().take(cursor).collect();
        let grapheme_indices: Vec<usize> = before_cursor
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(before_cursor.len()))
            .collect();

        let mut word_start = 0usize;
        let mut found_word = false;

        for i in (0..grapheme_indices.len()).rev() {
            if i == 0 {
                word_start = 0;
                break;
            }
            let grapheme = &before_cursor[grapheme_indices[i - 1]..grapheme_indices[i]];
            let is_word =
                grapheme.chars().next().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);

            if is_word {
                found_word = true;
            } else if found_word {
                word_start = i;
                break;
            }
        }

        if word_start < cursor {
            let word_start_byte = self.char_index_to_byte_index(word_start);
            let cursor_byte = self.char_index_to_byte_index(cursor);

            drop(content);
            let mut content_mut = self.content.borrow_mut_gen_only();
            let old_text = content_mut.clone();
            let new_content =
                format!("{}{}", &old_text[..word_start_byte], &old_text[cursor_byte..]);
            *content_mut = new_content;
            *self.selection.borrow_mut_gen_only() = Selection::from_cursor(word_start);
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        let selection = self.selection.borrow();
        if selection.is_empty() {
            return None;
        }

        let start = selection.start.min(selection.end);
        let end = selection.end.max(selection.start);
        let content = self.content.borrow();
        Some(content[start..end].to_string())
    }
}

#[derive(Clone)]
pub struct SharedTextEditor(Gc<TextEditor>);

impl Default for SharedTextEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedTextEditor {
    pub fn new() -> Self {
        Self(Gc::new(TextEditor::new()))
    }

    pub fn with_text(text: &str) -> Self {
        Self(Gc::new(TextEditor::with_text(text)))
    }

    pub fn editor(&self) -> &Gc<TextEditor> {
        &self.0
    }
}

unsafe impl Trace for SharedTextEditor {
    fn trace(&self, visitor: &mut impl rudo_gc::Visitor) {
        self.0.trace(visitor);
    }
}
