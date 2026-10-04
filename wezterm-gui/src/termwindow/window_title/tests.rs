use super::*;
use std::collections::HashMap;
use wezterm_term::Progress;

fn pane(title: &str, is_zoomed: bool) -> PaneInformation {
    PaneInformation {
        pane_id: 0,
        pane_index: 0,
        is_active: true,
        is_zoomed,
        has_unseen_output: false,
        left: 0,
        top: 0,
        width: 80,
        height: 24,
        pixel_width: 0,
        pixel_height: 0,
        title: title.to_string(),
        title_set_via_osc: false,
        user_vars: HashMap::new(),
        progress: Progress::default(),
    }
}

fn tab(tab_index: usize, active_pane: &PaneInformation) -> TabInformation {
    TabInformation {
        tab_id: 0,
        tab_index,
        is_active: true,
        is_last_active: false,
        active_pane: Some(active_pane.clone()),
        window_id: 0,
        tab_title: String::new(),
    }
}

#[test]
fn wid_006_a_preset_window_title_is_the_marker_before_the_default_title() {
    let pane = pane("✳ Claude Code", false);
    let tab = tab(1, &pane);
    assert_eq!(
        compute_window_title(None, Some("control_center"), Some(&tab), Some(&pane), 5),
        "[control_center] [2/5] ✳ Claude Code"
    );
}

#[test]
fn wid_007_a_custom_window_title_cannot_remove_the_marker() {
    let pane = pane("vim", false);
    let tab = tab(0, &pane);
    assert_eq!(
        compute_window_title(
            Some("custom".to_string()),
            Some("control_center"),
            Some(&tab),
            Some(&pane),
            3
        ),
        "[control_center] custom"
    );
}

#[test]
fn wid_008_a_window_without_a_marker_keeps_its_title_unchanged() {
    let pane = pane("vim", false);
    let tab = tab(1, &pane);
    assert_eq!(
        compute_window_title(None, None, Some(&tab), Some(&pane), 3),
        "[2/3] vim"
    );
}

#[test]
fn a_zoomed_pane_keeps_its_zoom_marker_after_the_preset_marker() {
    let pane = pane("vim", true);
    let tab = tab(0, &pane);
    assert_eq!(
        compute_window_title(None, Some("p"), Some(&tab), Some(&pane), 1),
        "[p] [Z] vim"
    );
}

#[test]
fn a_single_tab_window_shows_no_tab_index() {
    let pane = pane("vim", false);
    let tab = tab(0, &pane);
    assert_eq!(
        compute_window_title(None, None, Some(&tab), Some(&pane), 1),
        "vim"
    );
}

#[test]
fn a_marked_window_without_an_active_pane_is_titled_by_the_marker_alone() {
    assert_eq!(compute_window_title(None, Some("p"), None, None, 1), "[p]");
    assert_eq!(compute_window_title(None, None, None, None, 1), "");
}

#[test]
fn wid_005_a_mirror_window_of_a_preset_has_a_marked_title() {
    let mut mirror = mux::window::Window::new_with_preset(Some("default".to_string()), None, None);
    mirror.set_preset_from_server(Some("control_center".to_string()));
    let pane = pane("zsh", false);
    let tab = tab(0, &pane);
    let title = compute_window_title(None, mirror.get_preset(), Some(&tab), Some(&pane), 2);
    assert!(title.starts_with("[control_center] "), "{}", title);
}

#[test]
fn wid_009_the_recorded_title_is_the_one_handed_to_the_system() {
    let pane = pane("✳ Claude Code", false);
    let tab = tab(1, &pane);
    let mut last = LastWindowTitle::new(initial_window_title(Some("control_center")));
    assert_eq!(last.get(), "[control_center]");

    let title = compute_window_title(None, Some("control_center"), Some(&tab), Some(&pane), 5);
    let handed_to_system = last.record(title).to_string();

    assert_eq!(handed_to_system, "[control_center] [2/5] ✳ Claude Code");
    assert_eq!(last.get(), handed_to_system);
}

#[test]
fn d15_the_initial_title_of_a_preset_window_is_its_marker() {
    assert_eq!(
        initial_window_title(Some("control_center")),
        "[control_center]"
    );
    assert_eq!(initial_window_title(None), "wezterm");
}
