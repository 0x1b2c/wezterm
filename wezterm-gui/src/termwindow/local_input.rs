//! Client-side line editor that backs `InputMode::Local`.
//!
//! In Local mode, keystrokes never cross the network; they accumulate in a
//! per-pane `LineEditorState` and are echoed locally onto the pane's display.
//! Only when the user submits the line (Enter) is the buffered text written
//! to the PTY in a single `WriteToPane`-equivalent flush, eliminating
//! per-keystroke round-trip latency that dominates long-form input over a
//! mux connection.
//!
//! The buffer is a flat `Vec<char>` with a single `cursor` index in
//! characters. Embedded `\n` (from Shift-Enter) is permitted so that a
//! single Local-mode session can stage a multi-line message.

use mux::pane::Pane;
use mux::renderable::StableCursorPosition;
use std::sync::Arc;

/// Holds the in-flight buffer and edit cursor for a pane that is currently
/// in `InputMode::Local`. One instance lives per pane in `PaneState`.
#[derive(Debug, Default, Clone)]
pub struct LineEditorState {
    /// Editable buffer. Stored as `Vec<char>` rather than `String` so that
    /// cursor-by-character arithmetic (move-left, delete-before-cursor,
    /// etc.) does not have to rescan UTF-8 byte boundaries.
    buffer: Vec<char>,
    /// Cursor index into `buffer`, in characters. Always satisfies
    /// `cursor <= buffer.len()`.
    cursor: usize,
    /// Anchor cursor position captured when the pane entered Local mode.
    /// Local echo is rendered relative to this anchor; on submit we drop
    /// the anchor and let server output overwrite the area naturally.
    anchor: Option<StableCursorPosition>,
}

/// Outcome of feeding a keystroke through the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineEditorAction {
    /// The keystroke modified the buffer; the caller must re-render the
    /// editor's local echo.
    Updated,
    /// The user pressed Enter. The caller must transmit `payload` to the
    /// PTY and clear the editor.
    Submit { payload: String },
    /// The keystroke had no effect (e.g. cursor-left at start of buffer).
    NoOp,
}

impl LineEditorState {
    /// True if the buffer is empty and there is no pending state to render.
    pub fn is_idle(&self) -> bool {
        self.buffer.is_empty() && self.anchor.is_none()
    }

    /// Returns the current edit cursor offset (in characters) into the
    /// rendered buffer.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the current buffer contents as a string.
    pub fn buffer_text(&self) -> String {
        self.buffer.iter().collect()
    }

    /// Returns the anchor captured when Local mode was entered, or `None`
    /// if the editor has never rendered.
    pub fn anchor(&self) -> Option<StableCursorPosition> {
        self.anchor
    }

    /// Compute the visual position of the edit cursor relative to the
    /// anchor, walking `buffer[..cursor]` with the same line-break and
    /// width rules that `LocalPane::render_local_input` uses when emitting
    /// cells. The result is `(row_offset, col)` where `row_offset` counts
    /// rows below the anchor (0 means the same row) and `col` is the
    /// column on that row.
    ///
    /// `anchor_col` is the starting column on the first row (subsequent
    /// rows always start at column 0). `cols` is the pane width in cells.
    /// This is a pure function over `buffer`, `cursor`, and the two args,
    /// so it is easy to test without standing up a pane.
    pub fn visual_cursor_offset(&self, anchor_col: usize, cols: usize) -> (isize, usize) {
        let cols = cols.max(1);
        let mut row: isize = 0;
        let mut col = anchor_col;
        for &ch in &self.buffer[..self.cursor] {
            if ch == '\n' {
                row += 1;
                col = 0;
                continue;
            }
            let width = wezterm_term::unicode_column_width(&ch.to_string(), None).max(1);
            if col + width > cols {
                row += 1;
                col = 0;
            }
            col += width;
        }
        (row, col)
    }

    /// Capture the pane's current cursor as the anchor, if not already set.
    pub fn ensure_anchor(&mut self, pane: &Arc<dyn Pane>) {
        if self.anchor.is_none() {
            self.anchor = Some(pane.get_cursor_position());
        }
    }

    /// Reset the editor to the empty state. Called after submit and when
    /// leaving Local mode.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.anchor = None;
    }

    /// Insert a character at the cursor.
    pub fn insert_char(&mut self, c: char) -> LineEditorAction {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
        LineEditorAction::Updated
    }

    /// Insert a newline at the cursor (Shift-Enter).
    pub fn insert_newline(&mut self) -> LineEditorAction {
        self.insert_char('\n')
    }

    /// Insert a multi-character string at the cursor. Used for IME-composed
    /// text (`Key::Composed`) and any other path that delivers a chunk of
    /// text rather than a single keystroke. Returns `NoOp` if `s` is empty
    /// so the caller can short-circuit without scheduling a re-render.
    pub fn insert_str(&mut self, s: &str) -> LineEditorAction {
        let mut any = false;
        for ch in s.chars() {
            self.buffer.insert(self.cursor, ch);
            self.cursor += 1;
            any = true;
        }
        if any {
            LineEditorAction::Updated
        } else {
            LineEditorAction::NoOp
        }
    }

    /// Treat as user pressing Enter: produce the payload (buffer + trailing
    /// `\n`) and reset state.
    pub fn submit(&mut self) -> LineEditorAction {
        let mut payload: String = self.buffer.iter().collect();
        payload.push('\n');
        self.reset();
        LineEditorAction::Submit { payload }
    }

    /// Backspace: delete one character before the cursor.
    pub fn delete_before_cursor(&mut self) -> LineEditorAction {
        if self.cursor == 0 {
            return LineEditorAction::NoOp;
        }
        self.cursor -= 1;
        self.buffer.remove(self.cursor);
        LineEditorAction::Updated
    }

    /// Delete: delete one character at the cursor.
    pub fn delete_at_cursor(&mut self) -> LineEditorAction {
        if self.cursor >= self.buffer.len() {
            return LineEditorAction::NoOp;
        }
        self.buffer.remove(self.cursor);
        LineEditorAction::Updated
    }

    /// Ctrl-W: delete the word before the cursor. A "word" is a run of
    /// non-whitespace characters, optionally preceded by whitespace, in
    /// keeping with the readline convention.
    pub fn delete_word_before_cursor(&mut self) -> LineEditorAction {
        if self.cursor == 0 {
            return LineEditorAction::NoOp;
        }
        let new_cursor = self.find_word_left();
        self.buffer.drain(new_cursor..self.cursor);
        self.cursor = new_cursor;
        LineEditorAction::Updated
    }

    /// Ctrl-U: delete from cursor to start of the current line (i.e. back
    /// to the most recent embedded `\n` or buffer start).
    pub fn delete_to_line_start(&mut self) -> LineEditorAction {
        let line_start = self.current_line_start();
        if line_start == self.cursor {
            return LineEditorAction::NoOp;
        }
        self.buffer.drain(line_start..self.cursor);
        self.cursor = line_start;
        LineEditorAction::Updated
    }

    /// Ctrl-K: delete from cursor to end of the current line.
    pub fn delete_to_line_end(&mut self) -> LineEditorAction {
        let line_end = self.current_line_end();
        if line_end == self.cursor {
            return LineEditorAction::NoOp;
        }
        self.buffer.drain(self.cursor..line_end);
        LineEditorAction::Updated
    }

    /// Ctrl-A: move cursor to start of the current line.
    pub fn move_to_line_start(&mut self) -> LineEditorAction {
        let pos = self.current_line_start();
        if pos == self.cursor {
            return LineEditorAction::NoOp;
        }
        self.cursor = pos;
        LineEditorAction::Updated
    }

    /// Ctrl-E: move cursor to end of the current line.
    pub fn move_to_line_end(&mut self) -> LineEditorAction {
        let pos = self.current_line_end();
        if pos == self.cursor {
            return LineEditorAction::NoOp;
        }
        self.cursor = pos;
        LineEditorAction::Updated
    }

    /// Ctrl-B / Left: move cursor one character left.
    pub fn move_left(&mut self) -> LineEditorAction {
        if self.cursor == 0 {
            return LineEditorAction::NoOp;
        }
        self.cursor -= 1;
        LineEditorAction::Updated
    }

    /// Ctrl-F / Right: move cursor one character right.
    pub fn move_right(&mut self) -> LineEditorAction {
        if self.cursor >= self.buffer.len() {
            return LineEditorAction::NoOp;
        }
        self.cursor += 1;
        LineEditorAction::Updated
    }

    /// Ctrl-Shift-B: move cursor one word left.
    pub fn move_word_left(&mut self) -> LineEditorAction {
        let pos = self.find_word_left();
        if pos == self.cursor {
            return LineEditorAction::NoOp;
        }
        self.cursor = pos;
        LineEditorAction::Updated
    }

    /// Ctrl-Shift-F: move cursor one word right.
    pub fn move_word_right(&mut self) -> LineEditorAction {
        let pos = self.find_word_right();
        if pos == self.cursor {
            return LineEditorAction::NoOp;
        }
        self.cursor = pos;
        LineEditorAction::Updated
    }

    /// Ctrl-P / Up: move cursor up one logical line, preserving the column
    /// offset where possible.
    pub fn move_up(&mut self) -> LineEditorAction {
        let line_start = self.current_line_start();
        if line_start == 0 {
            return LineEditorAction::NoOp;
        }
        let column = self.cursor - line_start;
        // The character before line_start is the `\n` ending the previous
        // line; the line before that runs from prev_line_start to
        // line_start - 1.
        let prev_line_end = line_start - 1;
        let prev_line_start = self.buffer[..prev_line_end]
            .iter()
            .rposition(|c| *c == '\n')
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let prev_line_len = prev_line_end - prev_line_start;
        self.cursor = prev_line_start + column.min(prev_line_len);
        LineEditorAction::Updated
    }

    /// Ctrl-N / Down: move cursor down one logical line, preserving the
    /// column offset where possible.
    pub fn move_down(&mut self) -> LineEditorAction {
        let line_start = self.current_line_start();
        let line_end = self.current_line_end();
        if line_end >= self.buffer.len() {
            return LineEditorAction::NoOp;
        }
        let column = self.cursor - line_start;
        let next_line_start = line_end + 1;
        let next_line_end = self.buffer[next_line_start..]
            .iter()
            .position(|c| *c == '\n')
            .map(|idx| next_line_start + idx)
            .unwrap_or(self.buffer.len());
        let next_line_len = next_line_end - next_line_start;
        self.cursor = next_line_start + column.min(next_line_len);
        LineEditorAction::Updated
    }

    fn current_line_start(&self) -> usize {
        self.buffer[..self.cursor]
            .iter()
            .rposition(|c| *c == '\n')
            .map(|idx| idx + 1)
            .unwrap_or(0)
    }

    fn current_line_end(&self) -> usize {
        self.buffer[self.cursor..]
            .iter()
            .position(|c| *c == '\n')
            .map(|idx| self.cursor + idx)
            .unwrap_or(self.buffer.len())
    }

    /// Find the start of the word ending at or before `self.cursor`.
    fn find_word_left(&self) -> usize {
        let mut pos = self.cursor;
        while pos > 0 && self.buffer[pos - 1].is_whitespace() {
            pos -= 1;
        }
        while pos > 0 && !self.buffer[pos - 1].is_whitespace() {
            pos -= 1;
        }
        pos
    }

    /// Find the end of the word starting at or after `self.cursor`.
    fn find_word_right(&self) -> usize {
        let mut pos = self.cursor;
        while pos < self.buffer.len() && self.buffer[pos].is_whitespace() {
            pos += 1;
        }
        while pos < self.buffer.len() && !self.buffer[pos].is_whitespace() {
            pos += 1;
        }
        pos
    }
}

#[cfg(test)]
mod tests {
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
}
