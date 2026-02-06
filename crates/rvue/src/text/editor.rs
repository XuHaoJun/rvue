//! Text editing state for TextInput widgets.
//!
//! Provides text editing capabilities including:
//! - Text content management
//! - Selection/cursor position tracking
//! - IME composition support
//! - Basic text editing operations

use rudo_gc::{Gc, GcCell, Trace};

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
    pub cursor: usize,
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
        self.cursor = 0;
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

    pub fn content(&self) -> String {
        self.content.borrow().clone()
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

    pub fn set_composition(&self, text: &str, cursor: usize) {
        *self.composition.borrow_mut_gen_only() = ImeComposition { text: text.to_string(), cursor };
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

        {
            let mut content = self.content.borrow_mut_gen_only();
            let old_text = content.clone();
            let new_content = format!("{}{}", &old_text[..start], &old_text[end..]);
            *content = new_content;
        }

        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(start);
    }

    pub fn insert_text(&self, text: &str) {
        self.delete_selection();

        let cursor = self.selection.borrow().cursor();
        let new_text = text.to_string();
        let new_cursor = cursor + new_text.len();

        {
            let mut content = self.content.borrow_mut_gen_only();
            let old_text = content.clone();
            let new_content = format!("{}{}{}", &old_text[..cursor], new_text, &old_text[cursor..]);
            *content = new_content;
        }

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

        let mut content = self.content.borrow_mut_gen_only();
        let old_text = content.clone();
        let new_content = format!("{}{}", &old_text[..cursor - 1], &old_text[cursor..]);
        *content = new_content;

        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(cursor - 1);
    }

    pub fn delete(&self) {
        let selection = self.selection.borrow();
        if !selection.is_empty() {
            self.delete_selection();
            return;
        }

        let cursor = selection.cursor();
        let text_len = self.content.borrow().len();
        if cursor >= text_len {
            return;
        }

        let mut content = self.content.borrow_mut_gen_only();
        let old_text = content.clone();
        let new_content = format!("{}{}", &old_text[..cursor], &old_text[cursor + 1..]);
        *content = new_content;
    }

    pub fn move_cursor(&self, delta: isize) {
        let selection = self.selection.borrow();
        let cursor = selection.cursor();
        let text_len = self.content.borrow().len();
        let new_cursor = (cursor as isize + delta).clamp(0, text_len as isize) as usize;
        drop(selection);

        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(new_cursor);
    }

    pub fn move_to(&self, position: usize) {
        let text_len = self.content.borrow().len();
        let position = position.min(text_len);
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(position);
    }

    pub fn move_to_start(&self) {
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(0);
    }

    pub fn move_to_end(&self) {
        let len = self.content.borrow().len();
        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(len);
    }

    pub fn move_word_left(&self) {
        let cursor = self.selection.borrow().cursor();
        let text = self.content.borrow();
        let new_cursor = text[..cursor].word_boundary_left().unwrap_or(0);
        drop(text);

        *self.selection.borrow_mut_gen_only() = Selection::from_cursor(new_cursor);
    }

    pub fn move_word_right(&self) {
        let cursor = self.selection.borrow().cursor();
        let text = self.content.borrow();
        let text_len = text.len();
        let new_cursor = cursor.saturating_add(text[cursor..].word_boundary_left().unwrap_or(0));
        if new_cursor < text_len {
            let additional = text[new_cursor..].word_boundary_left().unwrap_or(0);
            let final_cursor = (new_cursor + additional).min(text_len);
            *self.selection.borrow_mut_gen_only() = Selection::from_cursor(final_cursor);
        } else {
            *self.selection.borrow_mut_gen_only() = Selection::from_cursor(text_len);
        }
    }

    pub fn delete_word(&self) {
        let cursor = self.selection.borrow().cursor();
        let text = self.content.borrow();
        let word_end = cursor.saturating_add(text[cursor..].word_boundary_left().unwrap_or(0));
        drop(text);

        if word_end > cursor {
            let mut content = self.content.borrow_mut_gen_only();
            let old_text = content.clone();
            let new_content = format!("{}{}", &old_text[..cursor], &old_text[word_end..]);
            *content = new_content;
            *self.selection.borrow_mut_gen_only() = Selection::from_cursor(cursor);
        }
    }

    pub fn backdelete_word(&self) {
        let selection = self.selection.borrow();
        if !selection.is_empty() {
            self.delete_selection();
            return;
        }

        let cursor = selection.cursor();
        let text = self.content.borrow();
        let word_start = text[..cursor].word_boundary_right().unwrap_or(0);
        drop(text);

        if cursor > word_start {
            let mut content = self.content.borrow_mut_gen_only();
            let old_text = content.clone();
            let new_content = format!("{}{}", &old_text[..word_start], &old_text[cursor..]);
            *content = new_content;
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

trait WordBoundary {
    fn word_boundary_left(&self) -> Option<usize>;
    fn word_boundary_right(&self) -> Option<usize>;
}

impl WordBoundary for str {
    fn word_boundary_left(&self) -> Option<usize> {
        if self.is_empty() {
            return Some(0);
        }

        let mut pos = 0usize;
        for (byte_idx, ch) in self.char_indices() {
            if ch.is_alphanumeric() || ch == '_' {
                pos = byte_idx;
            } else if pos > 0 {
                return Some(pos);
            }
        }
        Some(if pos > 0 { pos } else { self.len() })
    }

    fn word_boundary_right(&self) -> Option<usize> {
        if self.is_empty() {
            return Some(0);
        }

        let mut found_word = false;
        for (byte_idx, ch) in self.char_indices() {
            if ch.is_alphanumeric() || ch == '_' {
                found_word = true;
            } else if found_word {
                return Some(byte_idx);
            }
        }
        Some(self.len())
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
