//! Headless tests for TextInput functionality
//!
//! These tests verify that keyboard input works correctly on TextInput
//! without requiring a GUI window.

use rvue::text::editor::SharedTextEditor;

#[test]
fn test_text_input_editor_initialization() {
    let editor = SharedTextEditor::new();
    assert_eq!(editor.editor().content(), "");
    assert!(editor.editor().selection().is_empty());
    assert!(!editor.editor().is_composing());
}

#[test]
fn test_emoji_text_edit() {
    let editor = SharedTextEditor::new();

    editor.editor().insert_text("你好👋世界");
    assert_eq!(editor.editor().content(), "你好👋世界");

    editor.editor().backspace();
    assert_eq!(editor.editor().content(), "你好👋世");

    editor.editor().backspace();
    assert_eq!(editor.editor().content(), "你好👋");

    editor.editor().backspace();
    assert_eq!(editor.editor().content(), "你好");
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
    // "你好" is 6 bytes in UTF-8 (3 bytes per character)
    // cursor_range is in bytes per winit API
    editor.editor().set_composition("你好", Some((6, 6)));
    assert!(editor.editor().is_composing());
    assert_eq!(editor.editor().composition().text, "你好");
    assert_eq!(editor.editor().composition().cursor_start, 2);
    assert_eq!(editor.editor().composition().cursor_end, 2);

    // Clear composition
    editor.editor().clear_composition();
    assert!(!editor.editor().is_composing());

    // Commit composition
    editor.editor().set_composition("世界", Some((6, 6)));
    editor.editor().commit_composition();
    let _ = editor.editor().content(); // Drop the borrow
    assert_eq!(editor.editor().content(), "世界");
}

#[test]
fn test_text_input_composition_with_content() {
    let editor = SharedTextEditor::new();

    // Insert some content first
    editor.editor().insert_text("Hello");

    // Start composition at position 1 (after "H")
    editor.editor().set_composition("你好", Some((1, 1)));

    // The text with composition should show the composition inserted at cursor
    let full_text = editor.editor().text_with_composition();
    assert_eq!(full_text, "H你好ello");

    // The cursor offset should be at position 1 (after H)
    assert_eq!(editor.editor().composition_cursor_offset(), 1);
}

#[test]
fn test_text_input_composition_cursor_range() {
    let editor = SharedTextEditor::new();

    // Insert content
    editor.editor().insert_text("Hello");

    // Start composition with cursor at position 2 (after "He")
    editor.editor().set_composition("你好", Some((1, 1)));

    let composition = editor.editor().composition();
    assert_eq!(composition.cursor_start, 1);
    assert_eq!(composition.cursor_end, 1);
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
    use winit::keyboard::Key;

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
    if let Key::Character(ch) = &event.key {
        if !event.modifiers.alt && !event.modifiers.ctrl && !event.modifiers.logo {
            editor.editor().insert_text(ch);
        }
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

    if let Key::Named(NamedKey::Backspace) = &event.key {
        editor.editor().backspace();
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

        if let Key::Character(c) = &event.key {
            editor.editor().insert_text(c);
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

    if let Key::Named(NamedKey::Backspace) = &backspace_event.key {
        editor.editor().backspace();
    }

    assert_eq!(editor.editor().content(), "hell");
}

#[test]
fn test_text_input_ime_commit() {
    let editor = SharedTextEditor::new();

    // Simulate IME commit
    let event = rvue::event::types::ImeEvent::Commit("你好".to_string());

    if let rvue::event::types::ImeEvent::Commit(text) = event {
        editor.editor().insert_text(&text);
    }

    assert_eq!(editor.editor().content(), "你好");
}

#[test]
fn test_text_input_focus_and_keyboard_flow() {
    use rvue::event::types::{KeyState, KeyboardEvent};
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

    if let Key::Named(NamedKey::Backspace) = &key_event.key {
        editor.editor().backspace();
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
    let flags = gc_component.flags.read();

    assert!(
        flags.contains(ComponentFlags::ACCEPTS_POINTER),
        "TextInput should have ACCEPTS_POINTER flag"
    );
    assert!(
        flags.contains(ComponentFlags::ACCEPTS_FOCUS),
        "TextInput should have ACCEPTS_FOCUS flag"
    );
}

#[test]
fn test_mixed_text_edit() {
    let editor = SharedTextEditor::new();

    editor.editor().insert_text("Hello世界");
    assert_eq!(editor.editor().content(), "Hello世界");

    editor.editor().move_to(5);
    editor.editor().insert_text("你好");
    assert_eq!(editor.editor().content(), "Hello你好世界");
}

#[test]
fn test_chinese_text_backspace() {
    let editor = SharedTextEditor::new();

    editor.editor().insert_text("中文");
    assert_eq!(editor.editor().content(), "中文");

    editor.editor().backspace();
    assert_eq!(editor.editor().content(), "中");
}

#[test]
fn test_chinese_text_delete() {
    let editor = SharedTextEditor::new();

    editor.editor().insert_text("中文");
    assert_eq!(editor.editor().content(), "中文");

    editor.editor().move_to_start();
    editor.editor().delete();
    assert_eq!(editor.editor().content(), "文");
}
