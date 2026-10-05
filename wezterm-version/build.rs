fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // A file named `.tag`, written by the release workflow, holds this
    // fork's version for a release build. It does not affect the upstream
    // format version below, which scripts compare as a date.
    let mut fork_tag = None;
    if let Ok(tag) = std::fs::read("../.tag") {
        if let Ok(s) = String::from_utf8(tag) {
            fork_tag = Some(s.trim().to_string());
            println!("cargo:rerun-if-changed=../.tag");
        }
    }

    // The upstream format version is always derived from the git information
    let mut ci_tag = String::new();
    if let Ok(repo) = git2::Repository::discover(".") {
        if let Ok(ref_head) = repo.find_reference("HEAD") {
            let repo_path = repo.path().to_path_buf();

            if let Ok(resolved) = ref_head.resolve() {
                if let Some(name) = resolved.name() {
                    let path = repo_path.join(name);
                    if path.exists() {
                        println!(
                            "cargo:rerun-if-changed={}",
                            path.canonicalize().unwrap().display()
                        );
                    }
                }
            }
        }

        if let Ok(output) = std::process::Command::new("git")
            .args(&[
                "-c",
                "core.abbrev=8",
                "show",
                "-s",
                "--format=%cd-%h",
                "--date=format:%Y%m%d-%H%M%S",
            ])
            .output()
        {
            let info = String::from_utf8_lossy(&output.stdout);
            ci_tag = info.trim().to_string();
        }
    }

    let fork_version = fork_tag.unwrap_or_else(|| match short_head_hash() {
        Some(hash) => format!("1b2c-dev+{hash}"),
        None => "1b2c-dev".to_string(),
    });

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=WEZTERM_TARGET_TRIPLE={}", target);
    println!("cargo:rustc-env=WEZTERM_CI_TAG={}", ci_tag);
    println!("cargo:rustc-env=WEZTERM_FORK_VERSION={}", fork_version);
}

fn short_head_hash() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!hash.is_empty()).then_some(hash)
}
