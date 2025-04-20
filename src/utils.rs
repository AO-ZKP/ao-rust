use super::*;

#[mlua::lua_module(name = "utils")]
pub fn utils(lua: &Lua) -> LuaResult<LuaTable> {
    // Load the existing utils.lua
    let require: LuaFunction = lua.globals().get("require")?;
    let utils_table: LuaTable = require.call(".utils")?;

    // Override matchesSpec with Rust implementation
    utils_table.set("matchesSpec", lua.create_function(matches_spec)?)?;
    utils_table.set("matchesPattern", lua.create_function(matches_pattern)?)?;

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
                let matches_msg = msg_value.map_or(false, |v| {
                    matches_pattern(lua, (pattern.clone(), v, msg.clone())).unwrap_or(false)
                });
                let matches_tag = tag_value.map_or(false, |v| {
                    matches_pattern(lua, (pattern.clone(), v, msg.clone())).unwrap_or(false)
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

// fn matches_pattern(lua: &Lua, (pattern, value, msg): (LuaValue, LuaValue, LuaTable)) -> LuaResult<bool> {
//     matches_pattern_helper(lua, pattern, value, msg)
// }

fn matches_pattern(
    lua: &Lua,
    (pattern, value, msg): (LuaValue, LuaValue, LuaTable),
) -> LuaResult<bool> {
    // Case 1: If pattern is nil, return false
    if pattern == LuaValue::Nil {
        return Ok(false);
    }

    // Case 2: If pattern is the string "_", return true (wildcard)
    if let LuaValue::String(s) = &pattern {
        if s.to_str()? == "_" {
            return Ok(true);
        }
    }

    // Case 3: If pattern is a function, execute it with value and msg, return its truthiness
    if let LuaValue::Function(func) = &pattern {
        let result: LuaValue = func.call((value.clone(), msg))?;
        return match result {
            LuaValue::Nil => Ok(false),
            LuaValue::Boolean(b) => Ok(b),
            _ => Ok(true),
        };
    }

    // Case 4: If pattern is a string
    if let LuaValue::String(pat_str) = &pattern {
        let pat = pat_str.to_str()?;
        let val_str = value.to_string()?; // Coerce value to string, as Lua does implicitly
                                          // Check for special Lua pattern characters: [^$()%.[]*+?]
        if contains_special_chars(pat.as_ref()) {
            let string_mod: LuaTable = lua.globals().get("string")?;
            let match_fn: LuaFunction = string_mod.get("match")?;
            let result: LuaValue = match_fn.call((val_str, pat.as_ref()))?;
            if result != LuaValue::Nil {
                return Ok(true);
            }
        } else {
            // Exact string match
            if pat == val_str {
                return Ok(true);
            }
        }
    }

    // Case 5: If pattern is a table, recursively check sub-patterns
    if let LuaValue::Table(tbl) = &pattern {
        for pair in tbl.pairs::<LuaValue, LuaValue>() {
            let (_, sub_pattern) = pair?;
            if matches_pattern(lua, (sub_pattern, value.clone(), msg.clone()))? {
                return Ok(true);
            }
        }
    }

    // Default case: no match
    Ok(false)
}

fn contains_special_chars(s: &str) -> bool {
    s.contains(|c| {
        matches!(
            c,
            '^' | '$' | '(' | ')' | '%' | '.' | '[' | ']' | '*' | '+' | '?'
        )
    })
}
