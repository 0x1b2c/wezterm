use super::*;
use mux::window::Window;

fn presets() -> HashMap<WindowId, String> {
    let mut presets = HashMap::new();
    presets.insert(7, "control_center".to_string());
    presets
}

#[test]
fn wid_005_a_mirror_window_takes_the_preset_the_server_reports() {
    assert_eq!(
        preset_of_remote_window(&presets(), 7).as_deref(),
        Some("control_center")
    );
    assert_eq!(preset_of_remote_window(&presets(), 8), None);
}

#[test]
fn wid_005_the_primary_window_adopts_the_preset_of_the_remote_window_merged_into_it() {
    let mut primary = Window::new(Some("default".to_string()), None);
    primary.set_preset_from_server(preset_of_remote_window(&presets(), 7));
    assert_eq!(primary.get_preset(), Some("control_center"));
}

#[test]
fn an_empty_window_opened_by_a_preset_is_not_given_another_presets_window() {
    let window = Window::new_with_preset(
        Some("default".to_string()),
        None,
        Some("khala_declara".to_string()),
    );
    assert_eq!(
        primary_merge(&window, Some("hive".to_string())),
        PrimaryMerge::Skip
    );
    assert_eq!(primary_merge(&window, None), PrimaryMerge::Keep);
}

/// A client attached to a socket that nobody serves, so that
/// `process_pane_list` can be driven with hand-made pane lists.
struct Attached {
    inner: Arc<ClientInner>,
    _listener: wezterm_uds::UnixListener,
}

fn attached(name: &str) -> Attached {
    let path = crate::test_socket_path(name);
    let listener = wezterm_uds::UnixListener::bind(&path).unwrap();
    let dom = UnixDomain {
        name: name.into(),
        socket_path: Some(path),
        no_serve_automatically: true,
        ..Default::default()
    };
    let domain_id = alloc_domain_id();
    let mut ui = ConnectionUI::new_headless();
    let client = Client::new_unix_domain(Some(domain_id), &dom, false, &mut ui, true).unwrap();
    Attached {
        inner: Arc::new(ClientInner::new(domain_id, client, None, false)),
        _listener: listener,
    }
}

fn mux_for_test() -> promise::spawn::SimpleExecutor {
    config::use_test_configuration();
    let executor = promise::spawn::SimpleExecutor::new();
    Mux::set_mux(&Arc::new(Mux::new(None)));
    executor
}

/// A remote window as a server reports it: its preset and the ids of
/// its tabs, each holding a single pane with the same id.
struct RemoteWindow {
    id: WindowId,
    preset: Option<&'static str>,
    tabs: std::ops::Range<TabId>,
}

fn pane_list(windows: &[RemoteWindow]) -> ListPanesResponse {
    let size = TerminalSize::default();
    let mut response = ListPanesResponse {
        tabs: vec![],
        tab_titles: vec![],
        window_titles: HashMap::new(),
        window_presets: HashMap::new(),
    };
    for window in windows {
        response.window_titles.insert(window.id, String::new());
        if let Some(preset) = window.preset {
            response
                .window_presets
                .insert(window.id, preset.to_string());
        }
        for tab_id in window.tabs.clone() {
            response
                .tabs
                .push(mux::tab::PaneNode::Leaf(mux::tab::PaneEntry {
                    window_id: window.id,
                    tab_id,
                    pane_id: tab_id,
                    title: String::new(),
                    size,
                    working_dir: None,
                    is_active_pane: true,
                    is_zoomed_pane: false,
                    workspace: "default".into(),
                    cursor_pos: Default::default(),
                    physical_top: 0,
                    top_row: 0,
                    left_col: 0,
                    tty_name: None,
                    input_mode: Default::default(),
                }));
            response.tab_titles.push(String::new());
        }
    }
    response
}

fn sync(client: &Attached, windows: &[RemoteWindow], primary: Option<WindowId>) {
    ClientDomain::process_pane_list(Arc::clone(&client.inner), pane_list(windows), primary)
        .unwrap();
}

fn empty_window() -> WindowId {
    let builder = Mux::get().new_empty_window(None, None);
    *builder
}

fn preset_of(window_id: WindowId) -> Option<String> {
    Mux::get()
        .get_window(window_id)
        .unwrap()
        .get_preset()
        .map(str::to_string)
}

fn tab_count(window_id: WindowId) -> usize {
    Mux::get().get_window(window_id).unwrap().count_tabs()
}

/// The GUI mirrors its local mux server, whose window 2 was opened by the
/// `khala_declara` preset. Returns that window's local mirror.
fn mirror_of_local_server(local: &Attached) -> WindowId {
    sync(
        local,
        &[RemoteWindow {
            id: 0,
            preset: Some("control_center"),
            tabs: 0..2,
        }],
        Some(empty_window()),
    );
    sync(
        local,
        &[
            RemoteWindow {
                id: 0,
                preset: Some("control_center"),
                tabs: 0..2,
            },
            RemoteWindow {
                id: 2,
                preset: Some("khala_declara"),
                tabs: 2..5,
            },
        ],
        None,
    );
    local.inner.remote_to_local_window(2).unwrap()
}

#[test]
fn a_preset_window_keeps_its_preset_when_a_domain_without_presets_is_attached_from_it() {
    let _executor = mux_for_test();
    let local = attached("keeps-preset-local");
    let window = mirror_of_local_server(&local);

    // Attaching another domain from this window merges that domain's
    // first window into it.
    let other = attached("keeps-preset-other");
    sync(
        &other,
        &[RemoteWindow {
            id: 0,
            preset: None,
            tabs: 0..1,
        }],
        Some(window),
    );

    assert_eq!(preset_of(window).as_deref(), Some("khala_declara"));
    assert_eq!(tab_count(window), 4);
}

#[test]
fn the_empty_startup_window_takes_the_preset_of_the_remote_window_merged_into_it() {
    let _executor = mux_for_test();
    let local = attached("startup-window");
    let window = empty_window();

    sync(
        &local,
        &[RemoteWindow {
            id: 0,
            preset: Some("control_center"),
            tabs: 0..2,
        }],
        Some(window),
    );

    assert_eq!(preset_of(window).as_deref(), Some("control_center"));
    assert_eq!(tab_count(window), 2);
}

#[test]
fn a_remote_preset_window_gets_its_own_window_when_attached_from_a_window_with_tabs() {
    let _executor = mux_for_test();
    let local = attached("own-window-local");
    let window = mirror_of_local_server(&local);

    let other = attached("own-window-other");
    sync(
        &other,
        &[
            RemoteWindow {
                id: 0,
                preset: Some("hive"),
                tabs: 0..2,
            },
            RemoteWindow {
                id: 1,
                preset: None,
                tabs: 2..3,
            },
        ],
        Some(window),
    );

    let hive = other.inner.remote_to_local_window(0).unwrap();
    assert_ne!(hive, window);
    assert_eq!(preset_of(hive).as_deref(), Some("hive"));
    assert_eq!(tab_count(hive), 2);
    // The preset-less remote window is still merged into the window the
    // domain was attached from, which keeps its own preset.
    assert_eq!(other.inner.remote_to_local_window(1), Some(window));
    assert_eq!(preset_of(window).as_deref(), Some("khala_declara"));
    assert_eq!(tab_count(window), 4);
}
