use super::*;
use mux::domain::DomainId;
use mux::pane::{ForEachPaneLogicalLine, LogicalLine, WithPaneLines};
use parking_lot::{MappedMutexGuard, Mutex};
use rangeset::RangeSet;
use std::ops::Range;
use wezterm_term::color::ColorPalette;
use wezterm_term::{KeyCode, KeyModifiers, Line, MouseEvent, TerminalSize};

/// A pane whose only changing state is its keyboard encoding; everything
/// else `compute_changes` reads stays at its default, so any change it
/// reports comes from the encoding.
struct KeyboardPane {
    encoding: Mutex<KeyboardEncoding>,
}

impl KeyboardPane {
    fn new(encoding: KeyboardEncoding) -> Arc<Self> {
        Arc::new(Self {
            encoding: Mutex::new(encoding),
        })
    }

    fn set_encoding(&self, encoding: KeyboardEncoding) {
        *self.encoding.lock() = encoding;
    }
}

impl Pane for KeyboardPane {
    fn pane_id(&self) -> PaneId {
        1
    }
    fn get_cursor_position(&self) -> StableCursorPosition {
        StableCursorPosition::default()
    }
    fn get_current_seqno(&self) -> SequenceNo {
        0
    }
    fn get_changed_since(
        &self,
        _lines: Range<StableRowIndex>,
        _seqno: SequenceNo,
    ) -> RangeSet<StableRowIndex> {
        RangeSet::new()
    }
    /// `compute_changes` always sends the cursor's row, so every requested
    /// row must exist.
    fn get_lines(&self, lines: Range<StableRowIndex>) -> (StableRowIndex, Vec<Line>) {
        let first = lines.start;
        (first, lines.map(|_| Line::new(0)).collect())
    }
    fn with_lines_mut(&self, _lines: Range<StableRowIndex>, _with_lines: &mut dyn WithPaneLines) {
        unimplemented!()
    }
    fn for_each_logical_line_in_stable_range_mut(
        &self,
        _lines: Range<StableRowIndex>,
        _for_line: &mut dyn ForEachPaneLogicalLine,
    ) {
        unimplemented!()
    }
    fn get_logical_lines(&self, _lines: Range<StableRowIndex>) -> Vec<LogicalLine> {
        unimplemented!()
    }
    fn get_dimensions(&self) -> RenderableDimensions {
        RenderableDimensions::default()
    }
    fn get_title(&self) -> String {
        String::new()
    }
    fn send_paste(&self, _text: &str) -> anyhow::Result<()> {
        unimplemented!()
    }
    fn reader(&self) -> anyhow::Result<Option<Box<dyn std::io::Read + Send>>> {
        Ok(None)
    }
    fn writer(&self) -> MappedMutexGuard<'_, dyn std::io::Write> {
        unimplemented!()
    }
    fn resize(&self, _size: TerminalSize) -> anyhow::Result<()> {
        unimplemented!()
    }
    fn key_down(&self, _key: KeyCode, _mods: KeyModifiers) -> anyhow::Result<()> {
        unimplemented!()
    }
    fn key_up(&self, _key: KeyCode, _mods: KeyModifiers) -> anyhow::Result<()> {
        unimplemented!()
    }
    fn mouse_event(&self, _event: MouseEvent) -> anyhow::Result<()> {
        unimplemented!()
    }
    fn is_dead(&self) -> bool {
        false
    }
    fn palette(&self) -> ColorPalette {
        unimplemented!()
    }
    fn domain_id(&self) -> DomainId {
        1
    }
    fn is_mouse_grabbed(&self) -> bool {
        false
    }
    fn is_alt_screen_active(&self) -> bool {
        false
    }
    fn get_current_working_dir(&self, _policy: CachePolicy) -> Option<Url> {
        None
    }
    fn get_keyboard_encoding(&self) -> KeyboardEncoding {
        *self.encoding.lock()
    }
}

fn kitty_flags() -> KittyKeyboardFlags {
    KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
        | KittyKeyboardFlags::REPORT_EVENT_TYPES
        | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
}

/// The changes the client would get without any key press forcing them.
fn changes_without_input(
    per_pane: &mut PerPane,
    pane: &Arc<KeyboardPane>,
) -> Option<GetPaneRenderChangesResponse> {
    let pane: Arc<dyn Pane> = pane.clone();
    per_pane.compute_changes(&pane, None)
}

/// The changes sent back after a key press, which go out even when nothing
/// changed.
fn changes_after_input(
    per_pane: &mut PerPane,
    pane: &Arc<KeyboardPane>,
) -> GetPaneRenderChangesResponse {
    let pane: Arc<dyn Pane> = pane.clone();
    per_pane
        .compute_changes(&pane, Some(InputSerial::empty()))
        .expect("a key press always produces a change")
}

#[test]
fn a_pane_change_carries_the_kitty_flags_the_pane_reports() {
    let pane = KeyboardPane::new(KeyboardEncoding::Kitty(kitty_flags()));
    let mut per_pane = PerPane::default();

    let changes = changes_after_input(&mut per_pane, &pane);

    assert_eq!(changes.kitty_keyboard_flags, Some(kitty_flags()));
}

#[test]
fn a_change_of_keyboard_encoding_alone_is_sent_to_mux_clients() {
    let pane = KeyboardPane::new(KeyboardEncoding::Xterm);
    let mut per_pane = PerPane::default();
    assert!(changes_without_input(&mut per_pane, &pane).is_none());

    pane.set_encoding(KeyboardEncoding::Kitty(kitty_flags()));
    let enabled =
        changes_without_input(&mut per_pane, &pane).expect("turning kitty keyboard on is a change");
    assert_eq!(enabled.kitty_keyboard_flags, Some(kitty_flags()));

    pane.set_encoding(KeyboardEncoding::Xterm);
    let disabled = changes_without_input(&mut per_pane, &pane)
        .expect("turning kitty keyboard off is a change");
    assert_eq!(disabled.kitty_keyboard_flags, None);
}

#[test]
fn a_win32_keyboard_encoding_is_reported_to_mux_clients_as_xterm() {
    let pane = KeyboardPane::new(KeyboardEncoding::Kitty(kitty_flags()));
    let mut per_pane = PerPane::default();
    changes_after_input(&mut per_pane, &pane);

    pane.set_encoding(KeyboardEncoding::Win32);
    let win32 = changes_without_input(&mut per_pane, &pane)
        .expect("leaving kitty keyboard for win32 is a change");
    assert_eq!(win32.kitty_keyboard_flags, None);

    pane.set_encoding(KeyboardEncoding::Xterm);
    assert!(
        changes_without_input(&mut per_pane, &pane).is_none(),
        "win32 and xterm look the same to a mux client"
    );
}
