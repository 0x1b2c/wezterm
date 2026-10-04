//! Server-side helpers that materialize, list and tear down window presets.
//!
//! A preset is identified by its `config.window_presets` key. The window a
//! preset opens carries that key as its preset marker from the moment it is
//! created, and nothing rewrites the marker afterwards; a preset counts as
//! "running" when some mux window carries its marker. The window title plays
//! no part, since programs running in the window rewrite it freely.

use anyhow::{anyhow, Context};
use codec::WindowPresetStatus;
use config::keyassignment::{SpawnCommand, WindowPreset};
use mux::pane::Pane;
use mux::window::{Window, WindowId};
use mux::Mux;
use portable_pty::CommandBuilder;
use std::sync::Arc;

/// Returns the status of every preset declared in the current configuration,
/// in the order in which the names are iterated by the configuration map.
pub fn list_window_presets() -> Vec<WindowPresetStatus> {
    let config = config::configuration();
    let names: Vec<String> = config.window_presets.keys().cloned().collect();
    preset_statuses(&names, &window_markers())
}

/// The preset marker of one mux window, paired with the window's id.
type WindowMarker = (WindowId, Option<String>);

fn marker_of(window: &Window) -> WindowMarker {
    (window.window_id(), window.get_preset().map(str::to_string))
}

/// Snapshot the preset marker of every mux window.
fn window_markers() -> Vec<WindowMarker> {
    let mux = Mux::get();
    mux.iter_windows()
        .into_iter()
        .filter_map(|window_id| mux.get_window(window_id).map(|window| marker_of(&window)))
        .collect()
}

/// Report, for each preset name, whether some window carries its marker.
fn preset_statuses(names: &[String], markers: &[WindowMarker]) -> Vec<WindowPresetStatus> {
    names
        .iter()
        .map(|name| WindowPresetStatus {
            name: name.clone(),
            is_running: find_preset_window(markers, name).is_some(),
        })
        .collect()
}

/// Find the window opened by the preset `name`, if it is still open.
fn find_preset_window(markers: &[WindowMarker], name: &str) -> Option<WindowId> {
    markers
        .iter()
        .find(|(_, preset)| preset.as_deref() == Some(name))
        .map(|(window_id, _)| *window_id)
}

/// Materialize the preset identified by `name` into a freshly created mux
/// window with one tab per `SpawnCommand` in the preset.
///
/// Returns `Ok(None)` when a window opened by this preset already exists, so
/// callers can treat repeated invocations as idempotent.
pub async fn materialize_window_preset(name: &str) -> anyhow::Result<Option<WindowId>> {
    let preset = lookup_preset(name)?;

    if preset.tabs.is_empty() {
        anyhow::bail!("window preset `{name}` has no tabs configured");
    }

    if find_preset_window(&window_markers(), name).is_some() {
        return Ok(None);
    }

    let mux = Mux::get();
    let size = config::configuration().initial_size(0, None);

    // Spawn the first tab; this implicitly creates the window, which
    // carries the preset marker from the start.
    let first = &preset.tabs[0];
    let (_tab, pane, window_id) = mux
        .spawn_tab_or_window_with_preset(
            None,
            first.domain.clone(),
            command_builder_for(first)?,
            cwd_for(first),
            size,
            None,
            mux.active_workspace(),
            first.position.clone(),
            Some(name.to_string()),
        )
        .await
        .with_context(|| format!("spawning first tab of window preset `{name}`"))?;

    apply_send_text(first, &pane)
        .with_context(|| format!("send_text for first tab of window preset `{name}`"))?;

    // Subsequent tabs land in the same window.
    for (idx, tab) in preset.tabs.iter().enumerate().skip(1) {
        let (_tab, pane, _window_id) = mux
            .spawn_tab_or_window(
                Some(window_id),
                tab.domain.clone(),
                command_builder_for(tab)?,
                cwd_for(tab),
                size,
                None,
                mux.active_workspace(),
                None,
            )
            .await
            .with_context(|| {
                format!("spawning tab #{idx} of window preset `{name}` into window {window_id}")
            })?;

        apply_send_text(tab, &pane)
            .with_context(|| format!("send_text for tab #{idx} of window preset `{name}`"))?;
    }

    Ok(Some(window_id))
}

/// Honor the `send_text` field on a `SpawnCommand` by writing its bytes into
/// the freshly spawned pane's stdin. No-op when the field is unset.
fn apply_send_text(spawn: &SpawnCommand, pane: &Arc<dyn Pane>) -> anyhow::Result<()> {
    let Some(text) = spawn.send_text.as_ref() else {
        return Ok(());
    };
    pane.writer()
        .write_all(text.as_bytes())
        .map_err(anyhow::Error::from)
}

/// Tear down the running window backing the preset identified by `name`.
///
/// Returns an error when no window opened by that preset exists, so the
/// caller can surface a clear message rather than silently no-op.
pub fn kill_window_preset(name: &str) -> anyhow::Result<()> {
    let mux = Mux::get();
    let window_id = find_preset_window(&window_markers(), name)
        .ok_or_else(|| anyhow!("no running window matches preset `{name}`"))?;
    mux.kill_window(window_id);
    Ok(())
}

fn lookup_preset(name: &str) -> anyhow::Result<WindowPreset> {
    let config = config::configuration();
    config
        .window_presets
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow!("no window preset named `{name}`"))
}

fn command_builder_for(
    spawn: &config::keyassignment::SpawnCommand,
) -> anyhow::Result<Option<CommandBuilder>> {
    if spawn.args.is_none() && spawn.cwd.is_none() && spawn.set_environment_variables.is_empty() {
        return Ok(None);
    }

    let mut builder = match spawn.args.as_ref() {
        Some(args) => {
            if args.is_empty() {
                anyhow::bail!("SpawnCommand args must not be empty");
            }
            CommandBuilder::from_argv(args.iter().map(Into::into).collect())
        }
        None => CommandBuilder::new_default_prog(),
    };

    for (k, v) in &spawn.set_environment_variables {
        builder.env(k, v);
    }

    if let Some(cwd) = spawn.cwd.as_ref() {
        builder.cwd(cwd);
    }

    Ok(Some(builder))
}

fn cwd_for(spawn: &config::keyassignment::SpawnCommand) -> Option<String> {
    spawn
        .cwd
        .as_ref()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use mux::window::Window;

    fn preset_window(name: &str) -> Window {
        Window::new_with_preset(Some("default".to_string()), None, Some(name.to_string()))
    }

    fn plain_window() -> Window {
        Window::new_with_preset(Some("default".to_string()), None, None)
    }

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn wid_001_a_preset_stays_running_after_a_program_retitles_its_window() {
        let mut window = preset_window("control_center");
        window.set_title("✳ Claude Code");
        let markers = vec![marker_of(&window)];

        let statuses = preset_statuses(&names(&["control_center", "wezterm"]), &markers);

        assert_eq!(
            statuses,
            vec![
                WindowPresetStatus {
                    name: "control_center".to_string(),
                    is_running: true,
                },
                WindowPresetStatus {
                    name: "wezterm".to_string(),
                    is_running: false,
                },
            ]
        );
    }

    #[test]
    fn wid_002_opening_a_running_retitled_preset_finds_its_existing_window() {
        let mut window = preset_window("control_center");
        window.set_title("✳ Claude Code");
        let markers = vec![marker_of(&window)];

        assert_eq!(
            find_preset_window(&markers, "control_center"),
            Some(window.window_id())
        );
    }

    #[test]
    fn wid_003_closing_a_preset_targets_only_the_window_that_preset_opened() {
        let mut preset = preset_window("control_center");
        preset.set_title("✳ Claude Code");
        let mut impostor = plain_window();
        impostor.set_title("control_center");
        let markers = vec![marker_of(&impostor), marker_of(&preset)];

        assert_eq!(
            find_preset_window(&markers, "control_center"),
            Some(preset.window_id())
        );
    }

    #[test]
    fn a_window_titled_like_a_preset_is_not_counted_as_running() {
        let mut impostor = plain_window();
        impostor.set_title("control_center");
        let markers = vec![marker_of(&impostor)];

        assert_eq!(find_preset_window(&markers, "control_center"), None);
        assert!(!preset_statuses(&names(&["control_center"]), &markers)[0].is_running);
    }
}
