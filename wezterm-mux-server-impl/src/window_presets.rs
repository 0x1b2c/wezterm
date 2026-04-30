//! Server-side helpers that materialize, list and tear down window presets.
//!
//! A preset is identified by its `config.window_presets` key. The same key
//! doubles as the title applied to the materialized window, which is how a
//! preset is later recognized as "running" without keeping any extra state
//! outside the mux.

use anyhow::{anyhow, Context};
use codec::WindowPresetStatus;
use config::keyassignment::{SpawnCommand, WindowPreset};
use mux::pane::Pane;
use mux::window::WindowId;
use mux::Mux;
use portable_pty::CommandBuilder;
use std::sync::Arc;

/// Returns the status of every preset declared in the current configuration,
/// in the order in which the names are iterated by the configuration map.
pub fn list_window_presets() -> Vec<WindowPresetStatus> {
    let config = config::configuration();
    let running = running_preset_titles();

    config
        .window_presets
        .keys()
        .map(|name| WindowPresetStatus {
            name: name.clone(),
            is_running: running.contains(name.as_str()),
        })
        .collect()
}

/// Materialize the preset identified by `name` into a freshly created mux
/// window with one tab per `SpawnCommand` in the preset.
///
/// Returns `Ok(None)` when a window with the same title already exists, so
/// callers can treat repeated invocations as idempotent.
pub async fn materialize_window_preset(name: &str) -> anyhow::Result<Option<WindowId>> {
    let preset = lookup_preset(name)?;

    if preset.tabs.is_empty() {
        anyhow::bail!("window preset `{name}` has no tabs configured");
    }

    if find_window_by_title(name).is_some() {
        return Ok(None);
    }

    let mux = Mux::get();
    let size = config::configuration().initial_size(0, None);

    // Spawn the first tab; this implicitly creates the window.
    let first = &preset.tabs[0];
    let (_tab, pane, window_id) = mux
        .spawn_tab_or_window(
            None,
            first.domain.clone(),
            command_builder_for(first)?,
            cwd_for(first),
            size,
            None,
            mux.active_workspace(),
            first.position.clone(),
        )
        .await
        .with_context(|| format!("spawning first tab of window preset `{name}`"))?;

    if let Some(mut window) = mux.get_window_mut(window_id) {
        window.set_title(name);
    }

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
/// Returns an error when no window with that title exists, so the caller
/// can surface a clear message rather than silently no-op.
pub fn kill_window_preset(name: &str) -> anyhow::Result<()> {
    let mux = Mux::get();
    let window_id = find_window_by_title(name)
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

/// Collects the titles of every mux window. Used to determine whether a
/// preset is already running before materializing it.
fn running_preset_titles() -> std::collections::HashSet<String> {
    let mux = Mux::get();
    let mut titles = std::collections::HashSet::new();
    for window_id in mux.iter_windows() {
        if let Some(window) = mux.get_window(window_id) {
            titles.insert(window.get_title().to_string());
        }
    }
    titles
}

fn find_window_by_title(name: &str) -> Option<WindowId> {
    let mux = Mux::get();
    for window_id in mux.iter_windows() {
        if let Some(window) = mux.get_window(window_id) {
            if window.get_title() == name {
                return Some(window_id);
            }
        }
    }
    None
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
