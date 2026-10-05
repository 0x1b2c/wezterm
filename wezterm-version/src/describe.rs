//! Turns `git describe` output into this fork's version string. Shared by
//! build.rs, which runs git, and the crate's tests.

/// Builds the fork version from the output of
/// `git describe --tags --long --match '1b2c-*'`, which looks like
/// `1b2c-1.4.0-3-gabcd1234`, and the short hash of HEAD. A build exactly at a
/// release tag reports the release; a later build reports its distance from
/// it; a build with no release tag among its ancestors, as after an upstream
/// sync and before the next release, reports a dev version.
pub fn fork_version_from_describe(describe: Option<&str>, short_hash: Option<&str>) -> String {
    if let Some((tag, count, hash)) = describe.and_then(split_describe) {
        if count == "0" {
            return tag.to_string();
        }
        return format!("{tag}-dev.{count}+{hash}");
    }
    match short_hash {
        Some(hash) => format!("1b2c-dev+{hash}"),
        None => "1b2c-dev".to_string(),
    }
}

/// Splits `<tag>-<count>-g<hash>` into its three parts.
fn split_describe(describe: &str) -> Option<(&str, &str, &str)> {
    let mut parts = describe.trim().rsplitn(3, '-');
    let hash = parts.next()?.strip_prefix('g')?;
    let count = parts.next()?;
    let tag = parts.next()?;
    Some((tag, count, hash))
}

#[cfg(test)]
mod test {
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
}
