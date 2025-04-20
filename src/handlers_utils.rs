use super::*;
use crate::utils::matches_spec;

#[mlua::lua_module(name = "handlersUtils")]
pub fn handlers_utils(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("_version", "0.0.2")?;
    exports.set("hasMatchingTag", lua.create_function(has_matching_tag)?)?;
    exports.set(
        "hasMatchingTagOf",
        lua.create_function(has_matching_tag_of)?,
    )?;
    exports.set("hasMatchingData", lua.create_function(has_matching_data)?)?;
    exports.set("reply", lua.create_function(reply)?)?;
    exports.set("continue", lua.create_function(continue_fn)?)?;
    Ok(exports)
}

fn has_matching_tag(lua: &Lua, (name, value): (LuaString, LuaString)) -> LuaResult<LuaFunction> {
    let name = name.to_str()?.to_string();
    let value = value.to_str()?.to_string();
    let func = move |_: &Lua, msg: LuaTable| -> LuaResult<LuaValue> {
        let tags: LuaTable = msg.get("Tags")?;
        let tag_value: LuaValue = tags.get(name.as_str())?;
        if let LuaValue::String(s) = tag_value {
            Ok(LuaValue::Number(if s.to_str()? == value {
                -1.0
            } else {
                0.0
            }))
        } else {
            Ok(LuaValue::Number(0.0))
        }
    };
    lua.create_function(func)
}

fn has_matching_tag_of(lua: &Lua, (name, values): (LuaString, LuaTable)) -> LuaResult<LuaFunction> {
    let name = name.to_str()?.to_string();
    let values: Vec<String> = values
        .sequence_values()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|v: LuaValue| match v {
            LuaValue::String(s) => Ok(s.to_str()?.to_string()),
            _ => Err(LuaError::RuntimeError("values must be strings".to_string())),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let func = move |_lua: &Lua, msg: LuaTable| -> LuaResult<LuaValue> {
        let tags: LuaTable = msg.get("Tags")?;
        let tag_value: LuaValue = tags.get(name.as_str())?;
        if let LuaValue::String(s) = tag_value {
            let tag_str = s.to_str()?;
            if values.iter().any(|v| tag_str == v.as_str()) {
                Ok(LuaValue::Number(-1.0))
            } else {
                Ok(LuaValue::Number(0.0))
            }
        } else {
            Ok(LuaValue::Number(0.0))
        }
    };
    lua.create_function(func)
}

fn has_matching_data(lua: &Lua, value: LuaString) -> LuaResult<LuaFunction> {
    let value = value.to_str()?.to_string();
    let func = move |_lua: &Lua, msg: LuaTable| -> LuaResult<LuaValue> {
        let data: LuaValue = msg.get("Data")?;
        if let LuaValue::String(s) = data {
            Ok(LuaValue::Number(if s.to_str()? == value {
                -1.0
            } else {
                0.0
            }))
        } else {
            Ok(LuaValue::Number(0.0))
        }
    };
    lua.create_function(func)
}

fn reply(lua: &Lua, input: LuaValue) -> LuaResult<LuaFunction> {
    let input_clone = input.clone();
    let func = move |lua: &Lua, msg: LuaTable| -> LuaResult<()> {
        let reply_arg = match &input_clone {
            LuaValue::String(s) => {
                let table = lua.create_table()?;
                table.set("Data", s.clone())?;
                table
            }
            LuaValue::Table(t) => t.clone(),
            _ => {
                return Err(LuaError::RuntimeError(
                    "input must be string or table".to_string(),
                ))
            }
        };
        msg.call_method::<()>("reply", reply_arg)?;
        Ok(())
    };
    lua.create_function(func)
}

fn continue_fn(lua: &Lua, pattern: LuaValue) -> LuaResult<LuaFunction> {
    let pattern_clone = pattern.clone();
    let func = move |lua: &Lua, msg: LuaTable| -> LuaResult<LuaValue> {
        let match_result = matches_spec(lua, (msg.clone(), pattern_clone.clone()))?;
        match match_result {
            LuaValue::Nil => Ok(match_result),
            LuaValue::Boolean(b) if !b => Ok(match_result),
            LuaValue::Number(n) if n == 0.0 => Ok(match_result),
            LuaValue::String(ref s) if s.to_str()? == "skip" => Ok(match_result),
            _ => Ok(LuaValue::Number(1.0)),
        }
    };
    lua.create_function(func)
}
