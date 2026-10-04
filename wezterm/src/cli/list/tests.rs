use super::*;
use codec::ListPanesResponse;
use mux::tab::{PaneEntry, PaneNode};
use std::collections::HashMap;

fn pane(window_id: mux::window::WindowId, pane_id: mux::pane::PaneId) -> PaneNode {
    PaneNode::Leaf(PaneEntry {
        window_id,
        tab_id: pane_id,
        pane_id,
        title: "zsh".to_string(),
        size: TerminalSize::default(),
        working_dir: None,
        is_active_pane: true,
        is_zoomed_pane: false,
        workspace: "default".to_string(),
        cursor_pos: Default::default(),
        physical_top: 0,
        top_row: 0,
        left_col: 0,
        tty_name: None,
        input_mode: Default::default(),
    })
}

#[test]
fn wid_004_the_pane_list_reports_which_preset_each_window_belongs_to() {
    let mut window_presets = HashMap::new();
    window_presets.insert(1, "control_center".to_string());
    let panes = ListPanesResponse {
        tabs: vec![pane(1, 10), pane(1, 11), pane(2, 20)],
        tab_titles: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        window_titles: HashMap::new(),
        window_presets,
    };

    let json: Vec<serde_json::Value> = result_items(panes)
        .iter()
        .map(|item| serde_json::to_value(item).unwrap())
        .collect();

    assert_eq!(json[0]["window_preset"], "control_center");
    assert_eq!(json[1]["window_preset"], "control_center");
    assert_eq!(json[2]["window_preset"], serde_json::Value::Null);
}
