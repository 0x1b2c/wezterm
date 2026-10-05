pub fn wezterm_version() -> &'static str {
    // See build.rs
    env!("WEZTERM_CI_TAG")
}

pub fn wezterm_target_triple() -> &'static str {
    // See build.rs
    env!("WEZTERM_TARGET_TRIPLE")
}

/// This fork's own version, such as `1b2c-1.4.0`. It is shown to people;
/// `wezterm_version` keeps upstream's date-based format for the scripts and
/// tools that compare it.
pub fn wezterm_fork_version() -> &'static str {
    // See build.rs
    env!("WEZTERM_FORK_VERSION")
}

/// The text that `--version` prints after the program name: the fork
/// version followed by the mux protocol revision, the low 16 bits of the
/// codec version. Builds can only talk to each other when their protocol
/// revisions match.
pub fn format_version_line(fork_version: &str, codec_version: usize) -> String {
    format!("{fork_version} (protocol {})", codec_version & 0xffff)
}

/// `format_version_line` for this build, kept for the life of the process
/// because clap wants a `&'static str`.
pub fn version_line(codec_version: usize) -> &'static str {
    static LINE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    LINE.get_or_init(|| format_version_line(wezterm_fork_version(), codec_version))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn version_line_shows_the_fork_version_and_the_protocol_revision() {
        assert_eq!(
            format_version_line("1b2c-1.4.0", 0x1b2c_0001),
            "1b2c-1.4.0 (protocol 1)"
        );
    }
}
