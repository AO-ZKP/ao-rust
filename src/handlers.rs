use super::*;
use handlers_utils::handlers_utils;
/// Registers the `ao` module with Lua, initializing the `ao` table with fields and functions.
#[mlua::lua_module]
pub fn handlers(lua: &Lua) -> LuaResult<LuaTable> {
    // load the handlers module
    let require: LuaFunction = lua.globals().get("require")?;
    let handlers_lua: LuaTable = require.call(".handlers")?;
    // Set static fields
    handlers_lua.set("_version", "0.0.5")?;
    handlers_lua.set("utils", handlers_utils(lua)?)?;

    match lua.globals().get::<LuaTable>("Handlers") {
        Ok(handlers_table) => {
            handlers_table.set(
                "list",
                handlers_table
                    .get::<LuaTable>("list")
                    .unwrap_or_else(|_| lua.create_table().unwrap()),
            )?;
            handlers_table.set(
                "coroutines",
                handlers_table
                    .get::<LuaTable>("coroutines")
                    .unwrap_or_else(|_| lua.create_table().unwrap()),
            )?;
        }
        Err(_) => {
            let new_handlers = lua.create_table()?;
            new_handlers.set("list", lua.create_table()?)?;
            new_handlers.set("coroutines", lua.create_table()?)?;
            lua.globals().set("Handlers", new_handlers)?;
        }
    }

    handlers_lua.set("onceNonce", 0)?;

    Ok(handlers_lua)
}

fn _find_index_by_prop(array: &LuaTable, prop: &str, value: &LuaValue) -> LuaResult<Option<i32>> {
    for pair in array.pairs::<i32, LuaTable>() {
        let (index, object) = pair?;
        if object.get::<LuaValue>(prop)? == *value {
            return Ok(Some(index));
        }
    }
    Ok(None)
}
