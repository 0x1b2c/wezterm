//! Computes the title WezTerm hands to the operating system for a window.
//!
//! A window opened from a window preset carries a `[<preset name>] ` prefix
//! that is added outside the `format-window-title` result, so that user
//! scripts cannot remove it and external tools can rely on it to recognize
//! the window.

use super::{PaneInformation, TabInformation};

/// The title a system window starts with, before the first title update.
pub fn initial_window_title(preset: Option<&str>) -> String {
    match preset {
        Some(preset) => preset_prefix(preset).trim_end().to_string(),
        None => "wezterm".to_string(),
    }
}

/// The title to hand to the operating system: the `format-window-title`
/// result (or the default `[i/n] title` format when there is none), preceded
/// by the preset marker when the window has one.
pub fn compute_window_title(
    lua_title: Option<String>,
    preset: Option<&str>,
    active_tab: Option<&TabInformation>,
    active_pane: Option<&PaneInformation>,
    tabs_count: usize,
) -> String {
    let title =
        lua_title.unwrap_or_else(|| default_window_title(active_tab, active_pane, tabs_count));
    match preset {
        Some(preset) if title.is_empty() => initial_window_title(Some(preset)),
        Some(preset) => format!("{}{}", preset_prefix(preset), title),
        None => title,
    }
}

/// Remembers the title most recently handed to the operating system, which
/// is what `window:get_title()` reports to Lua.
#[derive(Debug, Clone, Default)]
pub struct LastWindowTitle(String);

impl LastWindowTitle {
    pub fn new(initial: String) -> Self {
        Self(initial)
    }

    /// Record `title` and return it, ready to hand to the operating system.
    pub fn record(&mut self, title: String) -> &str {
        self.0 = title;
        &self.0
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

fn preset_prefix(preset: &str) -> String {
    format!("[{}] ", preset)
}

fn default_window_title(
    active_tab: Option<&TabInformation>,
    active_pane: Option<&PaneInformation>,
    tabs_count: usize,
) -> String {
    let (Some(pos), Some(tab)) = (active_pane, active_tab) else {
        return String::new();
    };
    let zoomed = if pos.is_zoomed { "[Z] " } else { "" };
    if tabs_count == 1 {
        format!("{}{}", zoomed, pos.title)
    } else {
        format!(
            "{}[{}/{}] {}",
            zoomed,
            tab.tab_index + 1,
            tabs_count,
            pos.title
        )
    }
}

#[cfg(test)]
mod tests;
