use super::*;

#[test]
fn version_line_shows_the_fork_version_then_upstreams_protocol_version_and_the_forks_revision() {
    assert_eq!(
        format_version_line("1b2c-1.4.0", 0x1b2c_2e03),
        "1b2c-1.4.0 (protocol 46.3)"
    );
}
