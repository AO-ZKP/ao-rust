// Use necessary items from alloc for no_std compatibility

use super::*;

// Version constant
const VERSION: &str = "0.0.1";

// ANSI color codes for formatting
struct Colors {
    red: &'static str,
    green: &'static str,
    blue: &'static str,
    reset: &'static str,
}

const COLORS: Colors = Colors {
    red: "\x1b[31m",
    green: "\x1b[32m",
    blue: "\x1b[34m",
    reset: "\x1b[0m",
};

// Stringify module initialization
// Note: Using "stringify" as the name; adjust registration if ".stringify" is required
#[mlua::lua_module(name = "stringify")]
pub fn stringify(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("_version", VERSION)?;
    exports.set("isSimpleArray", lua.create_function(is_simple_array)?)?;
    exports.set("format", lua.create_function(format)?)?;
    Ok(exports)
}

// Check if a table is a simple array (consecutive numeric keys from 1)
fn is_simple_array(_lua: &Lua, tbl: LuaTable) -> LuaResult<bool> {
    let mut array_index = 1;
    for pair in tbl.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;
        match key {
            LuaValue::Integer(k) if k == array_index => {
                if !matches!(value, LuaValue::Number(_) | LuaValue::String(_)) {
                    return Ok(false);
                }
                array_index += 1;
            }
            _ => return Ok(false),
        }
    }
    Ok(true)
}

// Format a table for display
fn format(
    lua: &Lua,
    (tbl, indent, visited): (LuaTable, Option<i32>, Option<LuaTable>),
) -> LuaResult<String> {
    let indent = indent.unwrap_or(0);
    let to_indent = " ".repeat(indent as usize);
    let to_indent_child = " ".repeat((indent + 2) as usize);

    let mut result = Vec::new();
    let mut is_array = true;
    let mut array_index = 1;

    // Handle simple array case
    if is_simple_array(lua, tbl.clone())? {
        for value in tbl.sequence_values::<LuaValue>() {
            let v = value?;
            let formatted = match v {
                LuaValue::String(s) => format!("{}{}{}{}", COLORS.green, '"', s.to_str()?, COLORS.reset),
                _ => format!("{}{}{}", COLORS.blue, v.to_string()?, COLORS.reset),
            };
            result.push(formatted);
        }
        return Ok(format!("{{ {} }}", result.join(", ")));
    }

    // Handle non-array tables with potential circular references
    let visited = visited.unwrap_or_else(|| lua.create_table().unwrap());
    for pair in tbl.pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair?;
        if is_array {
            if let LuaValue::Integer(idx) = k {
                if idx == array_index {
                    array_index += 1;
                    let formatted = match v {
                        LuaValue::Table(ref t) => format(lua, (t.clone(), Some(indent + 2), Some(visited.clone())))?,
                        LuaValue::String(ref s) => format!("{}{}{}{}", COLORS.green, '"', s.to_str()?, COLORS.reset),
                        _ => format!("{}{}{}", COLORS.blue, v.to_string()?, COLORS.reset),
                    };
                    result.push(format!("{}{}", to_indent_child, formatted));
                } else {
                    is_array = false;
                    result.clear();
                }
            } else {
                is_array = false;
                result.clear();
            }
        }
        if !is_array {
            let formatted_key = format!("{}{}{}", COLORS.red, k.to_string()?, COLORS.reset);
            let formatted_value = match v {
                LuaValue::Table(ref t) => {
                    if visited.contains_key(t.clone())? {
                        "<circular reference>".to_string()
                    } else {
                        visited.set(t.clone(), true)?;
                        format(lua, (t.clone(), Some(indent + 2), Some(visited.clone())))?
                    }
                }
                LuaValue::String(ref s) => format!("{}{}{}{}", COLORS.green, '"', s.to_str()?, COLORS.reset),
                _ => format!("{}{}{}", COLORS.blue, v.to_string()?, COLORS.reset),
            };
            result.push(format!("{}{} = {}", to_indent_child, formatted_key, formatted_value));
        }
    }

    let (prefix, suffix, separator) = if is_array {
        ("{\n", format!("\n{}}}", to_indent), ",\n")
    } else {
        ("{\n ", format!("\n{}}}", to_indent), ",\n ")
    };
    Ok(format!("{}{}{}", prefix, result.join(separator), suffix))
}