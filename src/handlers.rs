use super::*;
use handlers_utils::handlers_utils;
/// Registers the `ao` module with Lua, initializing the `ao` table with fields and functions.
#[mlua::lua_module]
pub fn handlers(lua: &Lua) -> LuaResult<LuaTable> {
    // load the ao module
    let require: LuaFunction = lua.globals().get("require")?;
    let handlers_lua: LuaTable = require.call(".handlers")?;
    // Set static fields
    handlers_lua.set("_version", "0.0.5")?;
    handlers_lua.set("utils", handlers_utils(lua)?)?;

    Ok(handlers_lua)
}
