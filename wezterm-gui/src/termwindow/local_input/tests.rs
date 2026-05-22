use super::*;

#[test]
fn insert_and_backspace() {
    let mut e = LineEditorState::default();
    e.insert_char('h');
    e.insert_char('i');
    assert_eq!(e.buffer_text(), "hi");
    assert_eq!(e.cursor(), 2);
    e.delete_before_cursor();
    assert_eq!(e.buffer_text(), "h");
    assert_eq!(e.cursor(), 1);
}

#[test]
fn delete_word_before() {
    let mut e = LineEditorState::default();
    for c in "the quick fox".chars() {
        e.insert_char(c);
    }
    e.delete_word_before_cursor();
    assert_eq!(e.buffer_text(), "the quick ");
    e.delete_word_before_cursor();
    assert_eq!(e.buffer_text(), "the ");
}

#[test]
fn line_navigation_with_newlines() {
    let mut e = LineEditorState::default();
    for c in "first\nsecond\nthird".chars() {
        e.insert_char(c);
    }
    // cursor at end, on third line at col 5
    e.move_up();
    // now on second line at col 5
    assert_eq!(e.cursor(), "first\nsecon".len());
    e.move_up();
    // now on first line at col 5
    assert_eq!(e.cursor(), "first".len());
}

#[test]
fn ctrl_a_ctrl_e_within_line() {
    let mut e = LineEditorState::default();
    for c in "abc\ndef".chars() {
        e.insert_char(c);
    }
    e.move_to_line_start();
    assert_eq!(e.cursor(), "abc\n".len());
    e.move_to_line_end();
    assert_eq!(e.cursor(), "abc\ndef".len());
}

#[test]
fn submit_produces_payload_with_trailing_newline() {
    let mut e = LineEditorState::default();
    for c in "hello".chars() {
        e.insert_char(c);
    }
    match e.submit() {
        LineEditorAction::Submit { payload } => assert_eq!(payload, "hello\n"),
        other => panic!("unexpected action: {:?}", other),
    }
    assert!(e.is_idle());
}

#[test]
fn visual_cursor_offset_empty_buffer() {
    let e = LineEditorState::default();
    // Empty buffer: cursor sits at anchor, regardless of anchor_col.
    assert_eq!(e.visual_cursor_offset(0, 80), (0, 0));
    assert_eq!(e.visual_cursor_offset(7, 80), (0, 7));
}

#[test]
fn visual_cursor_offset_single_line_middle() {
    let mut e = LineEditorState::default();
    for c in "hello world".chars() {
        e.insert_char(c);
    }
    // Move cursor back so it sits in the middle of the buffer.
    for _ in 0..6 {
        e.move_left();
    }
    // cursor at index 5 (between "hello" and " world"), anchor at col 3:
    // 3 + 5 = 8, still on row 0.
    assert_eq!(e.visual_cursor_offset(3, 80), (0, 8));
}

#[test]
fn visual_cursor_offset_multi_line_via_newline() {
    let mut e = LineEditorState::default();
    // Simulate a Shift-Enter inserting a `\n` between two lines.
    for c in "abc\ndef".chars() {
        e.insert_char(c);
    }
    // Cursor at end (index 7): walked "abc" (col 4 from anchor_col=1),
    // hit `\n` (row=1, col=0), then "def" (col=3). Result: (1, 3).
    assert_eq!(e.visual_cursor_offset(1, 80), (1, 3));
}

#[test]
fn visual_cursor_offset_wraps_on_long_line() {
    let mut e = LineEditorState::default();
    // 12 chars, no embedded newlines.
    for c in "abcdefghijkl".chars() {
        e.insert_char(c);
    }
    // anchor_col=0, cols=5: rows fill at "abcde" (col 5 fits), then 'f'
    // wraps to row 1 col 0, fills to "fghij", then 'k' wraps to row 2,
    // ends after 'l' at row 2 col 2.
    assert_eq!(e.visual_cursor_offset(0, 5), (2, 2));
}
