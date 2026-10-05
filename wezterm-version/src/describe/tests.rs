use super::*;

#[test]
fn a_build_exactly_at_a_release_tag_reports_the_release() {
    assert_eq!(
        fork_version_from_describe(Some("1b2c-1.4.0-0-gabcd1234"), Some("abcd1234")),
        "1b2c-1.4.0"
    );
}

#[test]
fn a_build_after_a_release_reports_how_far_it_is_from_it() {
    assert_eq!(
        fork_version_from_describe(Some("1b2c-1.4.0-3-gabcd1234"), Some("abcd1234")),
        "1b2c-1.4.0-dev.3+abcd1234"
    );
}

#[test]
fn a_build_with_no_release_to_refer_to_reports_a_dev_version() {
    assert_eq!(
        fork_version_from_describe(None, Some("abcd1234")),
        "1b2c-dev+abcd1234"
    );
    assert_eq!(fork_version_from_describe(None, None), "1b2c-dev");
}
