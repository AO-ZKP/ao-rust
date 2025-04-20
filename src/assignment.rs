use super::*;
use crate::utils::matches_spec;

#[mlua::lua_module]
pub fn assignment(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("_version", "0.1.0")?;
    exports.set("init", lua.create_function(init)?)?;
    Ok(exports)
}

fn init(lua: &Lua, ao: LuaTable) -> LuaResult<()> {
    if ao.get::<Option<LuaTable>>("assignables")?.is_none() {
        ao.set("assignables", lua.create_table()?)?;
    }

    let ao_clone = ao.clone();
    let add_assignable = lua.create_function(move |lua, args: LuaMultiValue| {
        let mut args = args.into_iter();
        let first = args.next().unwrap_or(LuaValue::Nil);
        let second = args.next();
        let (name, match_spec) = match second {
            Some(val) => {
                let name = match first {
                    LuaValue::String(s) => s.to_str()?.to_string(),
                    _ => return Err(LuaError::RuntimeError("MatchSpec name MUST be a string".to_string())),
                };
                (Some(name), val)
            }
            None => (None, first),
        };
        let assignables: LuaTable = ao_clone.get("assignables")?;
        if let Some(name) = name {
            let value = LuaValue::String(lua.create_string(&name)?);
            if let Some(idx) = find_index_by_prop(lua, assignables.clone(), "name", value)? {
                let entry: LuaTable = assignables.get(idx)?;
                entry.set("pattern", match_spec)?;
            } else {
                let new_entry = lua.create_table()?;
                new_entry.set("pattern", match_spec)?;
                new_entry.set("name", name)?;
                assignables.set(assignables.len()? + 1, new_entry)?;
            }
        } else {
            let new_entry = lua.create_table()?;
            new_entry.set("pattern", match_spec)?;
            new_entry.set("name", LuaValue::Nil)?;
            assignables.set(assignables.len()? + 1, new_entry)?;
        }
        Ok(())
    })?;
    ao.set("addAssignable", add_assignable)?;

    let ao_clone = ao.clone();
    let remove_assignable = lua.create_function(move |_lua, name_or_index: LuaValue| {
        let assignables: LuaTable = ao_clone.get("assignables")?;
        match name_or_index {
            LuaValue::String(s) => {
                let value = LuaValue::String(s);
                if let Some(idx) = find_index_by_prop(_lua, assignables.clone(), "name", value)? {
                    assignables.raw_remove(idx)?;
                }
            }
            LuaValue::Integer(i) => {
                let len = assignables.len()? as i64;
                if i > 0 && i <= len {
                    assignables.raw_remove(i as usize)?;
                }
            }
            _ => return Err(LuaError::RuntimeError("index MUST be a number or string".to_string())),
        }
        Ok(())
    })?;
    ao.set("removeAssignable", remove_assignable)?;

    let ao_clone = ao.clone();
    let is_assignment = lua.create_function(move |_lua, msg: LuaTable| {
        let target: String = msg.get("Target")?;
        let ao_id: String = ao_clone.get("id")?;
        Ok(target != ao_id)
    })?;
    ao.set("isAssignment", is_assignment)?;

    let ao_clone = ao.clone();
    let is_assignable = lua.create_function(move |lua, msg: LuaTable| {
        let assignables: LuaTable = ao_clone.get("assignables")?;
        for i in 1..=assignables.len()? {
            let assignable: LuaTable = assignables.get(i)?;
            let pattern: LuaValue = assignable.get("pattern")?;
            let result: LuaValue = matches_spec(lua, (msg.clone(), pattern))?;
            let matches = match result {
                LuaValue::Nil => false,
                LuaValue::Boolean(b) => b,
                _ => true, // Any non-nil, non-false value is truthy
            };
            if matches {
                return Ok(true);
            }
        }
        Ok(false)
    })?;
    ao.set("isAssignable", is_assignable)?;

    Ok(())
}

fn find_index_by_prop(_lua: &Lua, array: LuaTable, prop: &str, value: LuaValue) -> LuaResult<Option<usize>> {
    for i in 1..=array.len()? {
        let entry: LuaTable = array.get(i)?;
        let entry_prop: LuaValue = entry.get(prop)?;
        if entry_prop == value {
            return Ok(Some(i.try_into().unwrap()));
        }
    }
    Ok(None)
}