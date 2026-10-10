use super::*;

#[cfg(unix)]
#[test]
fn a_test_socket_path_fits_the_unix_socket_limit_even_under_a_long_tmpdir() {
    // sun_path holds 104 bytes on macOS and 108 on Linux, its terminating
    // NUL included.
    let long_tmpdir = format!("/tmp/{}", "d".repeat(90));
    std::fs::create_dir_all(&long_tmpdir).unwrap();
    std::env::set_var("TMPDIR", &long_tmpdir);
    let path = test_socket_path("keeps-preset-other");
    let _ = std::fs::remove_dir(&long_tmpdir);
    let len = path.as_os_str().len();
    assert!(len < 104, "{} is {len} bytes", path.display());
}
