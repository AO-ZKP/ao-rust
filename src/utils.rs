use super::*;

#[mlua::lua_module(name = "utils")]
pub fn utils(lua: &Lua) -> LuaResult<LuaTable> {
    // Load the existing utils.lua
    let require: LuaFunction = lua.globals().get("require")?;
    let utils_table: LuaTable = require.call(".utils")?;

    // Override matchesSpec with Rust implementation
    utils_table.set("matchesSpec", lua.create_function(matches_spec)?)?;

    Ok(utils_table)
}

pub fn matches_spec(lua: &Lua, (msg, spec): (LuaTable, LuaValue)) -> LuaResult<LuaValue> {
    match spec {
        LuaValue::Function(func) => {
            let result: LuaValue = func.call(msg)?;
            Ok(result)
        }
        LuaValue::Table(table) => {
            for pair in table.pairs::<LuaString, LuaValue>() {
                let (key, pattern) = pair?;
                let key_str_owned = key.to_str()?; // Get the owned string
                let key_str = key_str_owned.as_ref(); // Convert to &str
                let msg_value: Option<LuaValue> = msg.get(key_str)?;
                let tags: Option<LuaTable> = msg.get("Tags")?;
                let tag_value: Option<LuaValue> = tags.and_then(|t| t.get(key_str).ok());
                if msg_value.is_none() && tag_value.is_none() {
                    return Ok(LuaValue::Boolean(false));
                }
                let require: LuaFunction = lua.globals().get("require")?;
                let utils: LuaTable = require.call("utils")?;
                let matches_pattern: LuaFunction = utils.get("matchesPattern")?;
                let matches_msg = msg_value.map_or(false, |v| {
                    matches_pattern
                        .call((pattern.clone(), v, msg.clone()))
                        .unwrap_or(false)
                });
                let matches_tag = tag_value.map_or(false, |v| {
                    matches_pattern
                        .call((pattern.clone(), v, msg.clone()))
                        .unwrap_or(false)
                });
                if !matches_msg && !matches_tag {
                    return Ok(LuaValue::Boolean(false));
                }
            }
            Ok(LuaValue::Boolean(true))
        }
        LuaValue::String(s) => {
            let action: Option<String> = msg.get("Action")?;
            let s_str = s.to_str()?;
            let matches = action.map_or(false, |a| a.as_str() == s_str.as_ref());
            Ok(LuaValue::Boolean(matches))
        }
        _ => Ok(LuaValue::Boolean(false)),
    }
}
