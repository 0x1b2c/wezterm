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
/// version followed by the mux protocol version, upstream's codec version
/// and the fork's protocol revision taken from the two low bytes of the
/// codec version, as in `(protocol 46.3)`. Builds can only talk to each
/// other when both numbers match.
pub fn format_version_line(fork_version: &str, codec_version: usize) -> String {
    format!(
        "{fork_version} (protocol {}.{})",
        (codec_version >> 8) & 0xff,
        codec_version & 0xff
    )
}

/// `format_version_line` for this build, kept for the life of the process
/// because clap wants a `&'static str`.
pub fn version_line(codec_version: usize) -> &'static str {
    static LINE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    LINE.get_or_init(|| format_version_line(wezterm_fork_version(), codec_version))
}

#[cfg(test)]
mod tests;
