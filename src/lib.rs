#![no_std]
use mlua::prelude::*;

fn add(_lua: &Lua, (a, b): (mlua::Integer, mlua::Integer)) -> LuaResult<mlua::Integer> {
    Ok(a + b)
}

#[mlua::lua_module]
fn addition(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("add", lua.create_function(add)?)?;
    Ok(exports)
}
