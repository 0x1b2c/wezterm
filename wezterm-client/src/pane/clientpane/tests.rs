use super::*;
use crate::client::Client;
use config::UnixDomain;
use mux::connui::ConnectionUI;
use mux::domain::alloc_domain_id;
use promise::spawn::SimpleExecutor;
use std::os::unix::net::UnixListener;

const REMOTE_PANE_ID: PaneId = 7;

/// A client pane attached to a socket that nobody serves, so that render
/// changes can be fed to it by hand as if the server had pushed them.
struct UnservedPane {
    pane: ClientPane,
    _listener: UnixListener,
    _executor: SimpleExecutor,
}

fn unserved_pane(name: &str) -> UnservedPane {
    config::use_test_configuration();
    let executor = SimpleExecutor::new();
    Mux::set_mux(&Arc::new(Mux::new(None)));

    let path = std::env::temp_dir().join(format!(
        "wezterm-clientpane-test-{}-{}",
        name,
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).unwrap();
    let domain = UnixDomain {
        name: name.into(),
        socket_path: Some(path),
        no_serve_automatically: true,
        ..Default::default()
    };
    let domain_id = alloc_domain_id();
    let mut ui = ConnectionUI::new_headless();
    let client = Client::new_unix_domain(Some(domain_id), &domain, false, &mut ui, true).unwrap();
    let inner = Arc::new(ClientInner::new(domain_id, client, None, false));
    let pane = ClientPane::new(&inner, 1, REMOTE_PANE_ID, TerminalSize::default(), "");

    UnservedPane {
        pane,
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
