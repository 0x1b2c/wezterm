pub mod client;
pub mod discovery;
pub mod domain;
pub mod pane;

/// A socket path of its own for a test, so tests can run side by side. Unix
/// tests use `/tmp` rather than `std::env::temp_dir()`: the per-user temporary
/// directory on macOS is about 50 bytes long already, and a socket path must
/// fit in the 104 bytes of `sun_path`.
#[cfg(test)]
pub(crate) fn test_socket_path(name: &str) -> std::path::PathBuf {
    #[cfg(unix)]
    let dir = std::path::PathBuf::from("/tmp");
    #[cfg(not(unix))]
    let dir = std::env::temp_dir();
    let path = dir.join(format!("wezterm-client-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

#[cfg(test)]
mod tests;
