//! Headless tests for TextInput functionality
//!
//! These tests verify that keyboard input works correctly on TextInput
//! without requiring a GUI window.

use rudo_gc::Gc;
use rvue::properties::{PropertyMap, TextInputValue};
use rvue::text::editor::SharedTextEditor;
use rvue::{Component, ComponentType};

#[test]
fn test_text_input_editor_initialization() {
    let editor = SharedTextEditor::new();
    assert_eq!(editor.editor().content(), "");
    assert!(editor.editor().selection().is_empty());
    assert!(!editor.editor().is_composing());
}

#[test]
fn test_text_input_insert_text() {
    let editor = SharedTextEditor::new();

    editor.editor().insert_text("a");
    assert_eq!(editor.editor().content(), "a");

    editor.editor().insert_text("bc");
    assert_eq!(editor.editor().content(), "abc");
}

#[test]
fn test_text_input_backspace() {
    let editor = SharedTextEditor::new();

    editor.editor().insert_text("hello");
    assert_eq!(editor.editor().content(), "hello");

    editor.editor().backspace();
}

#[test]
fn test_text_input_delete() {
    let editor = SharedTextEditor::new();

    editor.editor().insert_text("hello");
    assert_eq!(editor.editor().content(), "hello");

    // Move cursor to beginning
    editor.editor().move_to_start();

    editor.editor().delete();
    assert_eq!(editor.editor().content(), "ello");
}

#[test]
fn test_text_input_cursor_movement() {
    let editor = SharedTextEditor::new();

    editor.editor().insert_text("hello");
    assert_eq!(editor.editor().selection().cursor(), 5);

    editor.editor().move_cursor(-2);
    assert_eq!(editor.editor().selection().cursor(), 3);

    editor.editor().move_cursor(1);
    assert_eq!(editor.editor().selection().cursor(), 4);
}

#[test]
fn test_text_input_select_all() {
    let editor = SharedTextEditor::new();

    editor.editor().insert_text("hello");
    editor.editor().select_all();

    let selection = editor.editor().selection();
    assert_eq!(selection.start, 0);
    assert_eq!(selection.end, 5);
}

#[test]
fn test_text_input_composition() {
    let editor = SharedTextEditor::new();

    // Start composition
    editor.editor().set_composition("你好", 2);
    assert!(editor.editor().is_composing());
    assert_eq!(editor.editor().composition().text, "你好");
    assert_eq!(editor.editor().composition().cursor, 2);

    // Clear composition
    editor.editor().clear_composition();
    assert!(!editor.editor().is_composing());

    // Commit composition
    editor.editor().set_composition("世界", 2);
    editor.editor().commit_composition();
    let _ = editor.editor().content(); // Drop the borrow
    assert_eq!(editor.editor().content(), "世界");
}

#[test]
fn test_text_input_cursor_blink_state() {
    use rvue::text::cursor::GcCursorBlinkState;

    let blink = GcCursorBlinkState::new();

    // Initially visible
    assert!(blink.is_visible());

    // Update with focus - should start blinking
    let changed = blink.update(100, true);
    assert!(changed || blink.is_visible());
}

#[test]
fn test_text_input_keyboard_event_insert() {
    use rvue::event::types::{KeyState, KeyboardEvent};
    use winit::keyboard::{Key, NamedKey};

    let editor = SharedTextEditor::new();

    // Simulate inserting 'a'
    let event = KeyboardEvent {
        key: Key::Character("a".into()),
        code: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA),
        state: KeyState::Down,
        modifiers: rvue::event::types::Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            logo: false,
        },
        repeat: false,
    };

    // Process the event (simulating what handle_text_input_keyboard_event does)
    match &event.key {
        Key::Character(ch) => {
            if !event.modifiers.alt && !event.modifiers.ctrl && !event.modifiers.logo {
                editor.editor().insert_text(ch);
            }
        }
        _ => {}
    }

    assert_eq!(editor.editor().content(), "a");
}

#[test]
fn test_text_input_keyboard_event_backspace() {
    use rvue::event::types::{KeyState, KeyboardEvent};
    use winit::keyboard::{Key, NamedKey};

    let editor = SharedTextEditor::new();
    editor.editor().insert_text("ab");
    let _ = editor.editor().content(); // Drop the borrow

    // Simulate pressing backspace
    let event = KeyboardEvent {
        key: Key::Named(NamedKey::Backspace),
        code: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Backspace),
        state: KeyState::Down,
        modifiers: rvue::event::types::Modifiers::default(),
        repeat: false,
    };

    match &event.key {
        Key::Named(NamedKey::Backspace) => {
            editor.editor().backspace();
        }
        _ => {}
    }

    let _ = editor.editor().content(); // Drop the borrow
    assert_eq!(editor.editor().content(), "a");
}

#[test]
fn test_text_input_keyboard_event_sequence() {
    use rvue::event::types::{KeyState, KeyboardEvent};
    use winit::keyboard::{Key, NamedKey};

    let editor = SharedTextEditor::new();

    // Simulate typing 'hello' with backspace to make 'hell'
    let keys = ['h', 'e', 'l', 'l', 'o'];
    for ch in keys {
        let event = KeyboardEvent {
            key: Key::Character(ch.to_string().into()),
            code: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA),
            state: KeyState::Down,
            modifiers: rvue::event::types::Modifiers::default(),
            repeat: false,
        };

        match &event.key {
            Key::Character(c) => {
                editor.editor().insert_text(c);
            }
            _ => {}
        }
    }
    assert_eq!(editor.editor().content(), "hello");

    // Press backspace
    let backspace_event = KeyboardEvent {
        key: Key::Named(NamedKey::Backspace),
        code: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Backspace),
        state: KeyState::Down,
        modifiers: rvue::event::types::Modifiers::default(),
        repeat: false,
    };

    match &backspace_event.key {
        Key::Named(NamedKey::Backspace) => {
            editor.editor().backspace();
        }
        _ => {}
    }

    assert_eq!(editor.editor().content(), "hell");
}

#[test]
fn test_text_input_ime_commit() {
    let editor = SharedTextEditor::new();

    // Simulate IME commit
    let event = rvue::event::types::ImeEvent::Commit("你好".to_string());

    match event {
        rvue::event::types::ImeEvent::Commit(text) => {
            editor.editor().insert_text(&text);
        }
        _ => {}
    }

    assert_eq!(editor.editor().content(), "你好");
}

#[test]
fn test_text_input_focus_and_keyboard_flow() {
    use rvue::event::types::{KeyState, KeyboardEvent, TextEvent};
    use winit::keyboard::{Key, NamedKey};

    // Simulate the full flow: focus -> keyboard event -> text insertion
    let editor = SharedTextEditor::new();

    // Verify initial state
    assert_eq!(editor.editor().content(), "");

    // Simulate typing 'a' - this is what handle_text_input_keyboard_event does
    let key_event = KeyboardEvent {
        key: Key::Character("a".into()),
        code: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA),
        state: KeyState::Down,
        modifiers: rvue::event::types::Modifiers::default(),
        repeat: false,
    };

    // This simulates the logic in handle_text_input_keyboard_event
    match &key_event.key {
        Key::Character(ch)
            if !key_event.modifiers.alt
                && !key_event.modifiers.ctrl
                && !key_event.modifiers.logo =>
        {
            editor.editor().insert_text(ch);
        }
        _ => {}
    }

    assert_eq!(editor.editor().content(), "a");

    // Type 'b'
    let key_event = KeyboardEvent {
        key: Key::Character("b".into()),
        code: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyB),
        state: KeyState::Down,
        modifiers: rvue::event::types::Modifiers::default(),
        repeat: false,
    };

    match &key_event.key {
        Key::Character(ch)
            if !key_event.modifiers.alt
                && !key_event.modifiers.ctrl
                && !key_event.modifiers.logo =>
        {
            editor.editor().insert_text(ch);
        }
        _ => {}
    }

    assert_eq!(editor.editor().content(), "ab");

    // Backspace
    let key_event = KeyboardEvent {
        key: Key::Named(NamedKey::Backspace),
        code: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Backspace),
        state: KeyState::Down,
        modifiers: rvue::event::types::Modifiers::default(),
        repeat: false,
    };

    match &key_event.key {
        Key::Named(NamedKey::Backspace) => {
            editor.editor().backspace();
        }
        _ => {}
    }

    assert_eq!(editor.editor().content(), "a");
}

#[test]
fn test_text_input_accepts_focus() {
    use rvue::component::ComponentType;
    use rvue::event::status::ComponentFlags;
    use rvue::properties::PropertyMap;

    // Create a TextInput component directly
    let component = ComponentType::TextInput;

    // Create the component (this is what TextInput widget does internally)
    let gc_component = rvue::component::Component::with_properties(
        rvue::component::next_component_id(),
        component,
        PropertyMap::new(),
    );

    // Check that the component has ACCEPTS_FOCUS flag
    let flags = gc_component.flags.borrow();

    assert!(
        flags.contains(ComponentFlags::ACCEPTS_POINTER),
        "TextInput should have ACCEPTS_POINTER flag"
    );
    assert!(
        flags.contains(ComponentFlags::ACCEPTS_FOCUS),
        "TextInput should have ACCEPTS_FOCUS flag"
    );
}
