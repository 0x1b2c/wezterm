use super::super::*;
use crate::client::Client;
use config::UnixDomain;
use mux::connui::ConnectionUI;
use mux::domain::alloc_domain_id;
use std::path::{Path, PathBuf};

pub const REMOTE_PANE_ID: PaneId = 7;

/// A socket path of its own for each test, so tests can run side by side.
pub fn socket_path(name: &str) -> PathBuf {
    crate::test_socket_path(name)
}

/// A client pane for remote pane `REMOTE_PANE_ID`, connected to whatever
/// listens on `path`. A scheduler for spawned tasks must already be set up.
pub fn client_pane(name: &str, path: &Path) -> ClientPane {
    config::use_test_configuration();
    Mux::set_mux(&Arc::new(Mux::new(None)));

    let domain = UnixDomain {
        name: name.into(),
        socket_path: Some(path.to_path_buf()),
        no_serve_automatically: true,
        ..Default::default()
    };
    let domain_id = alloc_domain_id();
    let mut ui = ConnectionUI::new_headless();
    let client = Client::new_unix_domain(Some(domain_id), &domain, false, &mut ui, true).unwrap();
    let inner = Arc::new(ClientInner::new(domain_id, client, None, false));
    ClientPane::new(&inner, 1, REMOTE_PANE_ID, TerminalSize::default(), "")
}
