use super::super::*;
use super::fixture::{client_pane, socket_path, REMOTE_PANE_ID};
use promise::spawn::SimpleExecutor;
use wezterm_uds::UnixListener;

/// A client pane attached to a socket that nobody serves, so that render
/// changes can be fed to it by hand as if the server had pushed them.
struct UnservedPane {
    pane: ClientPane,
    _listener: UnixListener,
    _executor: SimpleExecutor,
}

fn unserved_pane(name: &str) -> UnservedPane {
    let executor = SimpleExecutor::new();
    let path = socket_path(name);
    let listener = UnixListener::bind(&path).unwrap();
    UnservedPane {
        pane: client_pane(name, &path),
        _listener: listener,
        _executor: executor,
    }
}

/// Render changes in which nothing but the kitty keyboard flags matter.
fn render_changes(kitty_keyboard_flags: Option<KittyKeyboardFlags>) -> Pdu {
    Pdu::GetPaneRenderChangesResponse(GetPaneRenderChangesResponse {
        pane_id: REMOTE_PANE_ID,
        mouse_grabbed: false,
        alt_screen_active: false,
        cursor_position: StableCursorPosition::default(),
        dimensions: RenderableDimensions::default(),
        dirty_lines: vec![],
        title: String::new(),
        working_dir: None,
        bonus_lines: SerializedLines::default(),
        input_serial: None,
        seqno: 0,
        kitty_keyboard_flags,
    })
}

#[test]
fn a_mux_client_pane_reports_the_kitty_flags_the_server_sent() {
    let unserved = unserved_pane("kitty-flags");
    let pane = &unserved.pane;
    let flags =
        KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES | KittyKeyboardFlags::REPORT_EVENT_TYPES;
    assert_eq!(pane.get_keyboard_encoding(), KeyboardEncoding::Xterm);

    smol::block_on(pane.process_unilateral(render_changes(Some(flags)))).unwrap();
    assert_eq!(pane.get_keyboard_encoding(), KeyboardEncoding::Kitty(flags));

    smol::block_on(pane.process_unilateral(render_changes(None))).unwrap();
    assert_eq!(pane.get_keyboard_encoding(), KeyboardEncoding::Xterm);
}
