use super::*;

/// Registers the `pretty` module with Lua, exporting `_version` and `tprint`.
#[mlua::lua_module(name = "pretty")]
pub fn pretty(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("_version", "0.0.1")?;
    exports.set("tprint", lua.create_function(tprint)?)?;
    Ok(exports)
}

/// Formats a Lua table into a readable string with indentation.
///
/// # Arguments
/// - `tbl`: The Lua table to format.
/// - `indent`: An optional indentation level (defaults to 0).
///
/// # Returns
/// A `LuaResult<String>` containing the formatted table representation.
///
/// # Behavior
/// - Iterates over all key-value pairs in the table.
/// - Indents each level with spaces based on the `indent` parameter.
/// - Recursively formats nested tables.
/// - Matches the output style of `pretty.tprint` from `pretty.lua`.
fn tprint(lua: &Lua, (tbl, indent): (LuaTable, Option<i32>)) -> LuaResult<String> {
    let indent = indent.unwrap_or(0);
    let mut output = String::new();

    // Iterate over all key-value pairs in the table
    for pair in tbl.pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair?;
        // Format the key with indentation and ": "
        let formatting = format!("{}{}: ", " ".repeat(indent as usize), k.to_string()?);

        if let LuaValue::Table(sub_tbl) = v {
            // For nested tables, add formatting and recurse
            output.push_str(&formatting);
            output.push('\n');
            let sub_output = tprint(lua, (sub_tbl, Some(indent + 1)))?;
            output.push_str(&sub_output);
        } else {
            // For non-table values, append the formatted key and value
            output.push_str(&formatting);
            output.push_str(&v.to_string()?);
            output.push('\n');
        }
    }

    Ok(output)
}
