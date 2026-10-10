use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A scratch directory under the system temp dir, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "wezterm-agent-link-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// Stand-in for an ssh-agent socket: any non-symlink file will do.
    fn socket(&self, name: &str) -> PathBuf {
        let path = self.path(name);
        std::fs::write(&path, b"").unwrap();
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).ok();
    }
}

#[test]
fn an_ordinary_agent_socket_becomes_the_target() {
    let dir = Scratch::new();
    let link = dir.path("agent.100");
    let first = dir.socket("first.sock");
    let second = dir.socket("second.sock");
    point_link_at(&first, &link).unwrap();

    assert_eq!(point_link_at(&second, &link).unwrap(), LinkUpdate::Updated);
    assert_eq!(std::fs::read_link(&link).unwrap(), second);
}

#[test]
fn a_symlink_chain_that_ends_at_an_agent_socket_becomes_the_target() {
    let dir = Scratch::new();
    let link = dir.path("agent.100");
    let socket = dir.socket("agent.sock");
    let other_mux_link = dir.path("agent.200");
    symlink_file(&socket, &other_mux_link).unwrap();

    assert_eq!(
        point_link_at(&other_mux_link, &link).unwrap(),
        LinkUpdate::Updated
    );
    assert_eq!(std::fs::read_link(&link).unwrap(), other_mux_link);
}

#[test]
fn the_link_itself_is_refused_and_the_working_target_is_kept() {
    let dir = Scratch::new();
    let link = dir.path("agent.100");
    let socket = dir.socket("agent.sock");
    point_link_at(&socket, &link).unwrap();

    assert_eq!(
        point_link_at(&link, &link).unwrap(),
        LinkUpdate::RefusedLoop
    );
    assert_eq!(std::fs::read_link(&link).unwrap(), socket);
}

#[test]
fn the_link_spelled_through_another_path_is_refused() {
    let dir = Scratch::new();
    let link = dir.path("agent.100");
    let socket = dir.socket("agent.sock");
    point_link_at(&socket, &link).unwrap();
    std::fs::create_dir(dir.path("sub")).unwrap();
    let roundabout = dir.path("sub").join("..").join("agent.100");

    assert_eq!(
        point_link_at(&roundabout, &link).unwrap(),
        LinkUpdate::RefusedLoop
    );
    assert_eq!(std::fs::read_link(&link).unwrap(), socket);
}

#[test]
fn a_link_that_leads_back_to_this_link_is_refused() {
    let dir = Scratch::new();
    let link = dir.path("agent.100");
    let socket = dir.socket("agent.sock");
    point_link_at(&socket, &link).unwrap();
    // A GUI started from a pane names its own agent.<pid> link, which
    // points at this mux's link.
    let gui_link = dir.path("agent.300");
    symlink_file(&link, &gui_link).unwrap();

    assert_eq!(
        point_link_at(&gui_link, &link).unwrap(),
        LinkUpdate::RefusedLoop
    );
    assert_eq!(std::fs::read_link(&link).unwrap(), socket);
}

#[test]
fn a_target_that_is_itself_a_symlink_loop_is_refused() {
    let dir = Scratch::new();
    let link = dir.path("agent.100");
    let socket = dir.socket("agent.sock");
    point_link_at(&socket, &link).unwrap();
    let stale = dir.path("agent.939");
    symlink_file(&stale, &stale).unwrap();

    assert_eq!(
        point_link_at(&stale, &link).unwrap(),
        LinkUpdate::RefusedLoop
    );
    assert_eq!(std::fs::read_link(&link).unwrap(), socket);
}
