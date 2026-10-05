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

// build.rs also loads this file through `#[path]`, which would resolve a
// plain `mod tests;` to `src/tests.rs`; the explicit path is the same from
// both places.
#[cfg(test)]
#[path = "describe/tests.rs"]
mod tests;
