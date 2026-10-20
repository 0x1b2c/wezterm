use super::*;
use std::time::Instant;
use wezterm_uds::UnixListener;

#[test]
fn a_version_mismatch_shows_both_codec_versions_in_hex() {
    let message = IncompatibleVersionError {
        version: "20261005-234300-3c8749cd".to_string(),
        codec_vers: 0x1b2c_0001,
    }
    .to_string();
    assert!(
        message.contains("(codec version 0x1b2c_0001)"),
        "{}",
        message
    );
    assert!(
        message.contains(&format!(
            "(codec version {}).",
            format_codec_version(CODEC_VERSION)
        )),
        "{}",
        message
    );
}

#[test]
fn a_just_spawned_server_that_starts_listening_after_a_few_seconds_is_still_reached() {
    let path = crate::test_socket_path("slow-start");
    let listen_path = path.clone();
    let (client_done, server_may_stop) = std::sync::mpsc::channel::<()>();
    let server = thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        let listener = UnixListener::bind(&listen_path).unwrap();
        // A connection completes from the backlog without an accept. The
        // listener stays up until the client has finished retrying, so a
        // retry delayed by a loaded machine still finds it.
        let _ = server_may_stop.recv_timeout(Duration::from_secs(30));
        drop(listener);
    });

    let started = Instant::now();
    let result = unix_connect_with_retry(&UnixTarget::Socket(path.clone()), true, None);
    let elapsed = started.elapsed();

    let _ = client_done.send(());
    server.join().unwrap();
    let _ = std::fs::remove_file(&path);
    assert!(
        result.is_ok(),
        "gave up after {elapsed:?}: {:#}",
        result.err().unwrap()
    );
}
