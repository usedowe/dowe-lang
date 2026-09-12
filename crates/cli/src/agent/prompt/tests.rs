use super::*;

fn typed(text: &str) -> Prompt {
    let mut prompt = Prompt::default();
    prompt.insert_text(text);
    prompt
}

#[test]
fn slash_filters_and_backspace_restores_matches() {
    let mut prompt = typed("/");
    assert_eq!(prompt.matches(), VISIBLE_COMMANDS);
    assert!(!prompt.matches().contains(&"/budget"));
    prompt.handle(Key::Char('l'), 80);
    assert_eq!(prompt.matches(), ["/login", "/logout"]);
    prompt.handle(Key::Char('o'), 80);
    prompt.handle(Key::Char('g'), 80);
    prompt.handle(Key::Char('i'), 80);
    assert_eq!(prompt.matches(), ["/login"]);
    prompt.handle(Key::Backspace, 80);
    assert_eq!(prompt.matches(), ["/login", "/logout"]);
    assert!(typed("hello /l").matches().is_empty());
    assert!(typed("/login extra").matches().is_empty());
}

#[test]
fn arrows_wrap_and_enter_submits_selected_command() {
    let mut prompt = typed("/l");
    prompt.handle(Key::ArrowUp, 80);
    assert_eq!(prompt.handle(Key::Enter, 80), Some(Some("/logout".into())));
    prompt.handle(Key::ArrowDown, 80);
    assert_eq!(prompt.handle(Key::Enter, 80), Some(Some("/login".into())));
    prompt.handle(Key::ArrowDown, 80);
    prompt.handle(Key::Char('o'), 80);
    assert_eq!(prompt.selected, 0);
    for command in VISIBLE_COMMANDS {
        assert_eq!(
            typed(command).handle(Key::Enter, 80),
            Some(Some(command.to_string()))
        );
    }
}

#[test]
fn advanced_commands_remain_explicit_without_entering_the_suggestion_list() {
    for command in [
        "/provider",
        "/sdd",
        "/plan inspect the project",
        "/research find the entrypoint",
        "/review inspect the change",
        "/memory status",
    ] {
        let mut prompt = typed(command);
        assert!(prompt.matches().is_empty(), "{command} should stay hidden");
        assert_eq!(prompt.handle(Key::Enter, 80), Some(Some(command.into())));
    }
}

#[test]
fn exit_aliases_are_not_supported_commands() {
    assert_eq!(typed("/quit").matches(), Vec::<&str>::new());
    assert_eq!(typed(":q").matches(), Vec::<&str>::new());
    assert_eq!(typed("/exit").matches(), ["/exit"]);
}

#[test]
fn escape_unknown_and_normal_input_preserve_text() {
    let mut prompt = typed("/l");
    prompt.handle(Key::Escape, 80);
    assert!(prompt.matches().is_empty());
    assert_eq!(prompt.handle(Key::Enter, 80), Some(Some("/l".into())));
    prompt.handle(Key::Char('o'), 80);
    assert_eq!(prompt.matches(), ["/login", "/logout"]);
    for text in ["", "hello", "/unknown", ":q"] {
        assert_eq!(typed(text).handle(Key::Enter, 80), Some(Some(text.into())));
    }
}

#[test]
fn unicode_editing_and_exit_keys() {
    let mut prompt = typed("a界🙂");
    prompt.handle(Key::ArrowLeft, 80);
    prompt.handle(Key::Backspace, 80);
    assert_eq!(prompt.value(), "a🙂");
    prompt.handle(Key::Del, 80);
    assert_eq!(prompt.value(), "a");
    prompt.handle(Key::Home, 80);
    prompt.handle(Key::Char('é'), 80);
    prompt.handle(Key::End, 80);
    assert_eq!(prompt.value(), "éa");
    assert_eq!(prompt.cursor, 2);
    assert_eq!(prompt.handle(Key::Char('\u{4}'), 80), None);
    assert_eq!(prompt.handle(Key::CtrlC, 80), Some(None));
    assert_eq!(typed("").handle(Key::Char('\u{4}'), 80), Some(None));
}

#[test]
fn multiline_input_wraps_scrolls_and_submits_without_loss() {
    let mut prompt = typed("one\ntwo\nthree");
    assert_eq!(prompt.value(), "one\ntwo\nthree");
    assert_eq!(
        prompt.input_view(6, MAX_INPUT_ROWS),
        InputView {
            rows: vec!["one", "two", "three"]
                .into_iter()
                .map(String::from)
                .collect(),
            cursor_row: 2,
            cursor_column: 5,
        }
    );
    assert_eq!(
        prompt.handle(Key::Enter, 6),
        Some(Some("one\ntwo\nthree".into()))
    );

    let prompt = typed("abcdefghijklmn");
    let view = prompt.input_view(4, 2);
    assert_eq!(view.rows, ["ijkl", "mn"]);
    assert_eq!(view.cursor_row, 1);
    assert_eq!(view.cursor_column, 2);

    let prompt = typed(&"x".repeat((MAX_INPUT_ROWS + 2) * 4));
    assert!(prompt.layout(4).rows.len() > MAX_INPUT_ROWS);
    assert_eq!(
        prompt.input_view(4, MAX_INPUT_ROWS).rows.len(),
        MAX_INPUT_ROWS
    );
}

#[test]
fn newline_editing_and_vertical_cursor_movement_use_visual_rows() {
    let mut prompt = typed("abcd\nxyz");
    prompt.handle(Key::Home, 4);
    assert_eq!(prompt.cursor, 5);
    prompt.handle(Key::ArrowUp, 4);
    assert_eq!(prompt.cursor, 0);
    prompt.handle(Key::End, 4);
    assert_eq!(prompt.cursor, 4);
    prompt.handle(Key::ArrowDown, 4);
    assert_eq!(prompt.cursor, 8);
    prompt.handle(Key::Char('\n'), 4);
    assert_eq!(prompt.value(), "abcd\nxyz\n");
}

#[test]
fn pasted_carriage_returns_normalize_to_single_line_breaks() {
    let mut prompt = Prompt::default();
    prompt.insert_text("first\r\nsecond\rthird");
    assert_eq!(prompt.value(), "first\nsecond\nthird");
}

#[test]
fn outline_bounds_unicode_labels_and_empty_viewports() {
    for width in 16..120 {
        for label in ["", " ⠋ Working… ", "界🙂".repeat(100).as_str()] {
            for border in input_outline(width, label) {
                assert_eq!(measure_text_width(&border), width);
            }
        }
    }
    assert_eq!(typed("界🙂").input_view(0, 1).cursor_column, 0);
}

#[test]
fn viewport_keeps_cursor_in_bounds() {
    let prompt = typed("hello界🙂world");
    for width in 1..20 {
        let view = prompt.input_view(width, MAX_INPUT_ROWS);
        assert!(
            view.rows
                .iter()
                .all(|line| measure_text_width(line) <= width)
        );
        assert!(view.cursor_column < width);
    }
}

#[test]
fn modified_enter_and_ctrl_j_are_line_breaks() {
    assert_eq!(
        prompt_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT)),
        Key::Char('\n')
    );
    assert_eq!(
        prompt_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)),
        Key::Char('\n')
    );
    assert_eq!(
        prompt_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL)),
        Key::Char('\n')
    );
    assert_eq!(
        prompt_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL)),
        Key::Char('\u{4}')
    );
}
