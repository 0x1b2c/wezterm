use super::*;

fn preset_name_seen_by_lua(preset: Option<&str>) -> Option<String> {
    let mux = Arc::new(Mux::new(None));
    Mux::set_mux(&mux);
    let builder = mux.new_empty_window_with_preset(None, None, preset.map(str::to_string));
    let window_id = *builder;

    let lua = Lua::new();
    lua.globals()
        .set("mux_window", MuxWindow(window_id))
        .unwrap();
    let name = lua
        .load("return mux_window:get_preset_name()")
        .eval::<Option<String>>()
        .unwrap();
    // Dropping the builder notifies subscribers through the main-thread
    // scheduler, which does not exist in a unit test.
    std::mem::forget(builder);
    Mux::shutdown();
    name
}

#[test]
fn get_preset_name_returns_the_preset_that_opened_the_window_or_nil() {
    // One test body: both cases share the process-wide Mux singleton.
    assert_eq!(
        preset_name_seen_by_lua(Some("work")),
        Some("work".to_string())
    );
    assert_eq!(preset_name_seen_by_lua(None), None);
}
