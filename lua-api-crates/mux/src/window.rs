use super::*;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard};

#[derive(Clone, Copy, Debug)]
pub struct MuxWindow(pub WindowId);

impl MuxWindow {
    pub fn resolve<'a>(
        &self,
        mux: &'a Arc<Mux>,
    ) -> mlua::Result<MappedRwLockReadGuard<'a, Window>> {
        mux.get_window(self.0)
            .ok_or_else(|| mlua::Error::external(format!("window id {} not found in mux", self.0)))
    }

    pub fn resolve_mut<'a>(
        &self,
        mux: &'a Arc<Mux>,
    ) -> mlua::Result<MappedRwLockWriteGuard<'a, Window>> {
        mux.get_window_mut(self.0)
            .ok_or_else(|| mlua::Error::external(format!("window id {} not found in mux", self.0)))
    }
}

impl UserData for MuxWindow {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, this, _: ()| {
            Ok(format!(
                "MuxWindow(mux_window_id:{}, pid:{})",
                this.0,
                unsafe { libc::getpid() }
            ))
        });
        methods.add_method("window_id", |_, this, _: ()| Ok(this.0));
        methods.add_async_method("gui_window", |lua, this, _: ()| async move {
            // Weakly bound to the gui module; mux cannot hard-depend
            // on wezterm-gui, but we can runtime resolve the appropriate module
            let wezterm_mod = get_or_create_module(lua, "wezterm")
                .map_err(|err| mlua::Error::external(format!("{err:#}")))?;
            let gui: mlua::Table = wezterm_mod.get("gui")?;
            let func: mlua::Function = gui.get("gui_window_for_mux_window")?;
            func.call_async::<_, mlua::Value>(this.0).await
        });
        methods.add_method("get_workspace", |_, this, _: ()| {
            let mux = get_mux()?;
            let window = this.resolve(&mux)?;
            Ok(window.get_workspace().to_string())
        });
        methods.add_method("set_workspace", |_, this, new_name: String| {
            let mux = get_mux()?;
            let mut window = this.resolve_mut(&mux)?;
            Ok(window.set_workspace(&new_name))
        });
        methods.add_async_method("spawn_tab", |_, this, spawn: SpawnTab| async move {
            spawn.spawn(this).await
        });
        methods.add_method("get_title", |_, this, _: ()| {
            let mux = get_mux()?;
            let window = this.resolve(&mux)?;
            Ok(window.get_title().to_string())
        });
        methods.add_method("set_title", |_, this, title: String| {
            let mux = get_mux()?;
            let mut window = this.resolve_mut(&mux)?;
            Ok(window.set_title(&title))
        });
        methods.add_method("get_preset_name", |_, this, _: ()| {
            let mux = get_mux()?;
            let window = this.resolve(&mux)?;
            Ok(window.get_preset().map(str::to_string))
        });
        methods.add_method("tabs", |_, this, _: ()| {
            let mux = get_mux()?;
            let window = this.resolve(&mux)?;
            Ok(window
                .iter_tabs()
                .map(|tab| MuxTab(tab.tab_id()))
                .collect::<Vec<MuxTab>>())
        });
        methods.add_method("tabs_with_info", |lua, this, _: ()| {
            let mux = get_mux()?;
            let window = this.resolve(&mux)?;
            let result = lua.create_table()?;
            let active_tab_idx = window.get_active_tab_idx();
            for (index, tab) in window.iter_tabs().enumerate() {
                let info = MuxTabInfo {
                    index,
                    is_active: index == active_tab_idx,
                };
                let info = luahelper::dynamic_to_lua_value(lua, info.to_dynamic())?;
                match &info {
                    LuaValue::Table(t) => {
                        t.set("tab", MuxTab(tab.tab_id()))?;
                    }
                    _ => {}
                }
                result.set(index + 1, info)?;
            }
            Ok(result)
        });
        methods.add_method("active_tab", |_, this, _: ()| {
            let mux = get_mux()?;
            let window = this.resolve(&mux)?;
            Ok(window.get_active_tab().map(|tab| MuxTab(tab.tab_id())))
        });
        methods.add_method("active_pane", |_, this, _: ()| {
            let mux = get_mux()?;
            let window = this.resolve(&mux)?;
            Ok(window
                .get_active_tab()
                .and_then(|tab| tab.get_active_pane().map(|pane| MuxPane(pane.pane_id()))))
        });
    }
}

#[cfg(test)]
mod test {
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
}
