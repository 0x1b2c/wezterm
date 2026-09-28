fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Gate on the target rather than the host: this build script is compiled
    // for the host, so `#[cfg(windows)]` would skip resource embedding when
    // cross-compiling a Windows binary from another OS.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        use std::io::Write;
        use std::path::Path;

        let repo_dir = std::env::current_dir()
            .ok()
            .and_then(|cwd| cwd.parent().map(|p| p.to_path_buf()))
            .unwrap();
        let windows_dir = repo_dir.join("assets").join("windows");

        let rcfile_name = Path::new(&std::env::var_os("OUT_DIR").unwrap()).join("resource.rc");
        let mut rcfile = std::fs::File::create(&rcfile_name).unwrap();
        write!(
            rcfile,
            r#"
#include <winres.h>
1 RT_MANIFEST "{win}{sep}console.manifest"
"#,
            win = windows_dir.display().to_string().replace("\\", "\\\\"),
            sep = RC_PATH_SEP,
        )
        .unwrap();
        drop(rcfile);

        // Obtain MSVC environment so that the rc compiler can find the right headers.
        // https://github.com/nabijaczleweli/rust-embed-resource/issues/11#issuecomment-603655972
        let target = std::env::var("TARGET").unwrap();
        if let Some(tool) = cc::windows_registry::find_tool(target.as_str(), "cl.exe") {
            for (key, value) in tool.env() {
                std::env::set_var(key, value);
            }
        }
        embed_resource::compile(rcfile_name);
    }
}

// Path separator written into the generated .rc file. Windows hosts keep the
// escaped backslash; other hosts (cross-compiling via windres) use a slash.
#[cfg(windows)]
const RC_PATH_SEP: &str = "\\\\";
#[cfg(not(windows))]
const RC_PATH_SEP: &str = "/";
