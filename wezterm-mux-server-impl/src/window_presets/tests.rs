use super::*;
use config::keyassignment::SpawnTabDomain;
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

const HOME: &str = "/home/me";

fn tab_in(cwd: &str) -> SpawnCommand {
    SpawnCommand {
        cwd: Some(PathBuf::from(cwd)),
        ..SpawnCommand::default()
    }
}

fn tab_without_directory() -> SpawnCommand {
    SpawnCommand::default()
}

fn directories_of(tabs: &[SpawnCommand]) -> Vec<Option<PathBuf>> {
    preset_tab_directories(tabs, Path::new(HOME))
}

fn dir(path: &str) -> Option<PathBuf> {
    Some(PathBuf::from(path))
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

#[test]
fn a_preset_tab_directory_starting_with_tilde_slash_opens_under_home() {
    assert_eq!(
        directories_of(&[tab_in("~/projects/veil")]),
        vec![dir("/home/me/projects/veil")]
    );
}

#[test]
fn a_preset_tab_directory_of_a_lone_tilde_opens_in_home() {
    assert_eq!(
        directories_of(&[tab_in("~"), tab_in("~/")]),
        vec![dir(HOME), dir(HOME)]
    );
}

#[test]
fn a_preset_tab_directory_naming_another_users_home_is_used_as_written() {
    assert_eq!(
        directories_of(&[tab_in("~other/projects"), tab_in("~other")]),
        vec![dir("~other/projects"), dir("~other")]
    );
}

#[test]
fn an_absolute_preset_tab_directory_is_used_as_written() {
    assert_eq!(
        directories_of(&[tab_in("/srv/projects/~/veil")]),
        vec![dir("/srv/projects/~/veil")]
    );
}

#[test]
fn a_preset_tab_without_a_directory_opens_where_the_previous_tab_opened() {
    assert_eq!(
        directories_of(&[tab_in("/srv/veil"), tab_without_directory()]),
        vec![dir("/srv/veil"), dir("/srv/veil")]
    );
}

#[test]
fn a_first_preset_tab_without_a_directory_keeps_the_default_directory() {
    assert_eq!(
        directories_of(&[tab_without_directory(), tab_without_directory()]),
        vec![None, None]
    );
}

#[test]
fn preset_tabs_without_a_directory_all_follow_the_last_one_that_named_one() {
    assert_eq!(
        directories_of(&[
            tab_without_directory(),
            tab_in("/srv/site"),
            tab_without_directory(),
            tab_without_directory(),
            tab_in("/srv/veil"),
            tab_without_directory(),
        ]),
        vec![
            None,
            dir("/srv/site"),
            dir("/srv/site"),
            dir("/srv/site"),
            dir("/srv/veil"),
            dir("/srv/veil"),
        ]
    );
}

#[test]
fn a_preset_tab_inherits_the_expanded_directory_of_the_previous_tab() {
    assert_eq!(
        directories_of(&[tab_in("~/projects/veil"), tab_without_directory()]),
        vec![dir("/home/me/projects/veil"), dir("/home/me/projects/veil")]
    );
}

#[test]
fn a_preset_tab_after_one_that_runs_a_program_still_inherits_its_directory() {
    let editor = SpawnCommand {
        args: Some(vec!["nvim".to_string(), ".".to_string()]),
        ..tab_in("/srv/veil")
    };

    assert_eq!(
        directories_of(&[editor, tab_without_directory()]),
        vec![dir("/srv/veil"), dir("/srv/veil")]
    );
}

#[test]
fn a_preset_tab_after_one_on_another_domain_still_inherits_its_directory() {
    let remote = SpawnCommand {
        domain: SpawnTabDomain::DomainName("remote".to_string()),
        ..tab_in("/srv/veil")
    };

    assert_eq!(
        directories_of(&[remote, tab_without_directory()]),
        vec![dir("/srv/veil"), dir("/srv/veil")]
    );
}
