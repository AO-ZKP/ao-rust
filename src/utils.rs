use super::*;

const VERSION: &str = "0.0.1";

#[mlua::lua_module(name = "utils")]
pub fn utils(lua: &Lua) -> LuaResult<LuaTable> {
    // Load the existing utils.lua
    let require: LuaFunction = lua.globals().get("require")?;
    let utils_table: LuaTable = require.call(".utils")?;

    utils_table.set("_version", VERSION)?;
    utils_table.set("matchesSpec", lua.create_function(matches_spec)?)?;
    utils_table.set("matchesPattern", lua.create_function(matches_pattern)?)?;
    utils_table.set("isArray", lua.create_function(is_array)?)?;
    utils_table.set("curry", lua.create_function(curry)?)?;
    utils_table.set("concat", lua.create_function(concat)?)?;
    utils_table.set("reduce", lua.create_function(reduce)?)?;

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

fn is_array(_lua: &Lua, table: LuaValue) -> LuaResult<bool> {
    if let LuaValue::Table(tbl) = table {
        let mut max_index: i64 = 0;
        for pair in tbl.pairs::<LuaValue, LuaValue>() {
            let (k, _) = pair?;
            if let LuaValue::Number(n) = k {
                let int_n = n.floor();
                if n < 1.0 || n != int_n {
                    return Ok(false); // Non-integer or negative key
                }
                max_index = max_index.max(int_n as i64);
            } else {
                return Ok(false); // Non-numeric key
            }
        }
        // Compare max_index with table length
        let len = tbl.len()?.try_into().unwrap_or(0);
        return Ok(max_index == len);
    }
    Ok(false)
}

fn curry(lua: &Lua, (fn_val, arity): (LuaFunction, Option<i32>)) -> LuaResult<LuaFunction> {
    // Determine arity if not provided
    let arity = arity.unwrap_or_else(|| {
        let debug: LuaTable = lua.globals().get("debug").unwrap();
        let getinfo: LuaFunction = debug.get("getinfo").unwrap();
        let info: LuaTable = getinfo.call((fn_val.clone(), "u")).unwrap();
        info.get("nparams").unwrap_or(0)
    });
    if arity < 2 {
        return Ok(fn_val);
    }

    // Create curried function
    let curried = move |lua: &Lua, args: LuaMultiValue| {
        let args: Vec<LuaValue> = args.into_vec();
        if args.len() >= arity as usize {
            fn_val.call(args)
        } else {
            let original_fn = fn_val.clone();
            let captured_args = args.clone();
            let new_arity = arity - args.len() as i32;
            let inner_fn = lua.create_function(
                move |_lua, inner_args: LuaMultiValue| -> Result<LuaMultiValue, LuaError> {
                    let mut combined_args = captured_args.clone();
                    combined_args.extend(inner_args);
                    original_fn.call(combined_args)
                },
            )?;
            curry(lua, (inner_fn, Some(new_arity)))
        }
    };

    lua.create_function(curried)
}

fn concat(lua: &Lua, a: LuaTable) -> LuaResult<LuaFunction> {
    // Verify that a is an array
    if !is_array(lua, LuaValue::Table(a.clone()))? {
        return Err(LuaError::RuntimeError(
            "first argument should be a table that is an array".to_string(),
        ));
    }

    let concat_inner = move |lua: &Lua, b: LuaTable| {
        // Verify that b is an array
        if !is_array(lua, LuaValue::Table(a.clone()))? {
            return Err(LuaError::RuntimeError(
                "second argument should be a table that is an array".to_string(),
            ));
        }

        let result = lua.create_table()?;
        let mut index = 1;
        // Copy elements from a
        for i in 1..=a.len()? {
            let value: LuaValue = a.get(i)?;
            result.set(index, value)?;
            index += 1;
        }
        // Copy elements from b
        for i in 1..=b.len()? {
            let value: LuaValue = b.get(i)?;
            result.set(index, value)?;
            index += 1;
        }
        Ok(result)
    };

    lua.create_function(concat_inner)
}

fn reduce(lua: &Lua, fn_val: LuaFunction) -> LuaResult<LuaFunction> {
    let fn_val = fn_val.clone(); // Clone it here so we can move a clone into the closure
    let reduce_initial = move |lua: &Lua, initial: LuaValue| {
        let fn_val = fn_val.clone(); // Clone again for the inner closure
        let reduce_table = move |lua: &Lua, t: LuaTable| {
            // Verify that t is an array
            if !is_array(lua, LuaValue::Table(t.clone()))? {
                return Err(LuaError::RuntimeError(
                    "third argument should be a table that is an array".to_string(),
                ));
            }

            let mut result = initial.clone();
            for pair in t.pairs::<LuaValue, LuaValue>() {
                let (k, v) = pair?;
                if result == LuaValue::Nil {
                    result = v.clone();
                } else {
                    result = fn_val.call((result.clone(), v, k))?;
                }
            }
            Ok(result)
        };
        lua.create_function(reduce_table)
    };
    lua.create_function(reduce_initial)
}
