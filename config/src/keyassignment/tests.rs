use super::*;

fn preset(lua_tabs: &str) -> anyhow::Result<WindowPreset> {
    let lua = mlua::Lua::new();
    let value: mlua::Value = lua
        .load(&format!("return {{ tabs = {lua_tabs} }}"))
        .eval()?;
    Ok(luahelper::from_lua_value_dynamic(value)?)
}

fn tab(cwd: Option<&str>, send_text: Option<&str>) -> SpawnCommand {
    SpawnCommand {
        cwd: cwd.map(PathBuf::from),
        send_text: send_text.map(str::to_string),
        ..SpawnCommand::default()
    }
}

#[test]
fn a_lua_table_with_positional_and_named_entries_converts_to_an_object_keeping_both() {
    let lua = mlua::Lua::new();
    let value: mlua::Value = lua.load("return { '~/x', cmd = 'ls' }").eval().unwrap();
    let mut expected = std::collections::BTreeMap::new();
    expected.insert(Value::I64(1), Value::String("~/x".to_string()));
    expected.insert(
        Value::String("cmd".to_string()),
        Value::String("ls".to_string()),
    );
    assert_eq!(
        luahelper::lua_value_to_dynamic(value).unwrap(),
        Value::Object(expected.into())
    );
}

#[test]
fn a_lua_table_holding_only_a_sequence_still_converts_to_an_array() {
    let lua = mlua::Lua::new();
    let value: mlua::Value = lua.load("return { 'a', 'b' }").eval().unwrap();
    assert_eq!(
        luahelper::lua_value_to_dynamic(value).unwrap(),
        Value::Array(
            vec![
                Value::String("a".to_string()),
                Value::String("b".to_string())
            ]
            .into()
        )
    );
}

#[test]
fn a_preset_tab_written_as_just_a_path_opens_in_that_directory() {
    let preset = preset("{ { '~/devel/x' } }").unwrap();
    assert_eq!(preset.tabs, vec![tab(Some("~/devel/x"), None)]);
}

#[test]
fn a_preset_tab_with_cmd_types_the_command_and_presses_enter() {
    let preset = preset("{ { cmd = 'ssh build-host' } }").unwrap();
    assert_eq!(preset.tabs, vec![tab(None, Some("ssh build-host\n"))]);
}

#[test]
fn a_preset_tab_with_a_path_and_cmd_gets_both() {
    let preset = preset("{ { '~/projects/app', cmd = 'make watch' } }").unwrap();
    assert_eq!(
        preset.tabs,
        vec![tab(Some("~/projects/app"), Some("make watch\n"))]
    );
}

#[test]
fn preset_tabs_in_the_shorthand_equal_the_same_tabs_spelled_out() {
    let short = preset(
        "{
            { '~/projects/site' },
            {},
            { cmd = 'bun run dev --host' },
            { '~/projects/app', cmd = 'make watch', domain = 'DefaultDomain' },
        }",
    )
    .unwrap();
    let long = preset(
        "{
            { cwd = '~/projects/site' },
            {},
            { send_text = 'bun run dev --host\\n' },
            { cwd = '~/projects/app', send_text = 'make watch\\n', domain = 'DefaultDomain' },
        }",
    )
    .unwrap();
    assert_eq!(short, long);
    // A tab that names no directory keeps cwd empty, so it inherits the
    // previous tab's directory when the preset is materialized.
    assert_eq!(short.tabs[1], tab(None, None));
    assert_eq!(short.tabs[2].cwd, None);
}

#[test]
fn a_preset_tab_naming_its_directory_both_ways_is_rejected() {
    let err = preset("{ { '~/a', cwd = '~/b' } }").unwrap_err();
    assert!(format!("{err:#}").contains("cwd"), "{:#}", err);
}

#[test]
fn a_preset_tab_with_both_cmd_and_send_text_is_rejected() {
    let err = preset("{ { cmd = 'ls', send_text = 'ls\\n' } }").unwrap_err();
    assert!(format!("{err:#}").contains("send_text"), "{:#}", err);
}

#[test]
fn a_preset_tab_with_more_than_one_positional_value_is_rejected() {
    assert!(preset("{ { '~/a', '~/b' } }").is_err());
}
