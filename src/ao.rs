use super::*;

/// Registers the `ao` module with Lua by loading `ao.lua` via `require` and replacing only implemented functions.
#[mlua::lua_module]
pub fn ao(lua: &Lua) -> LuaResult<LuaTable> {
    let require: LuaFunction = lua.globals().get("require")?;
    let ao_lua: LuaTable = require.call(".ao")?;

    // Replace only the fully implemented ao.log function
    ao_lua.set("log", lua.create_function(log)?)?;

    Ok(ao_lua)
}

/// Logs a message to the output, replacing the Lua version of ao.log.
fn log(lua: &Lua, (ao, txt): (LuaTable, String)) -> LuaResult<()> {
    let outbox: LuaTable = ao.get("outbox")?;
    let output: LuaValue = outbox.get("Output")?;

    let output_table = match output {
        LuaValue::String(s) => {
            let new_table = lua.create_table()?;
            new_table.set(1, s.to_str()?.to_string())?;
            outbox.set("Output", new_table.clone())?;
            new_table
        }
        LuaValue::Table(t) => t,
        _ => {
            let new_table = lua.create_table()?;
            outbox.set("Output", new_table.clone())?;
            new_table
        }
    };

    output_table.push(txt)?;
    Ok(())
}
