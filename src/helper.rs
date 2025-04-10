use mlua::prelude::*;

// Print function that pushes output to Lua's stdout via the Lua runtime
pub fn print(lua: &Lua, msg: &str) -> LuaResult<()> {
    let print_fn = lua.globals().get::<LuaFunction>("print")?;
    print_fn.call::<()>(msg)?; // Corrected: only specify return type as ()
    Ok(())
}

// Helper to create a table from key-value pairs
pub fn create_table_from_pairs<K, V>(lua: &Lua, pairs: impl IntoIterator<Item = (K, V)>) -> LuaResult<LuaTable>
where
    K: IntoLua,
    V: IntoLua,
{
    let table = lua.create_table()?;
    for (k, v) in pairs {
        table.set(k, v)?;
    }
    Ok(table)
}