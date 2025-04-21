use super::*;
use crate::utils::is_array;

/// Registers the `ao` module with Lua, initializing the `ao` table with fields and functions.
#[mlua::lua_module]
pub fn ao(lua: &Lua) -> LuaResult<LuaTable> {
    // load the ao module
    let require: LuaFunction = lua.globals().get("require")?;
    let ao_lua: LuaTable = require.call(".ao")?;

    let oldao: Option<LuaTable> = match lua.globals().get::<LuaTable>("ao") {
        Ok(table) => Some(table),
        Err(_) => Some(lua.create_table()?),
    };

    // Set static fields
    ao_lua.set("_version", "0.0.6")?;

    // Set fields with defaults from `oldao` if available
    if let Some(old_ao) = &oldao {
        ao_lua.set(
            "id",
            old_ao
                .get("id")
                .unwrap_or(LuaValue::String(lua.create_string("")?)),
        )?;
        ao_lua.set(
            "_module",
            old_ao
                .get("_module")
                .unwrap_or(LuaValue::String(lua.create_string("")?)),
        )?;
        ao_lua.set(
            "authorities",
            old_ao
                .get("authorities")
                .unwrap_or_else(|_| lua.create_table().unwrap()),
        )?;
        ao_lua.set("reference", old_ao.get("reference").unwrap_or(0))?;
        ao_lua.set(
            "outbox",
            old_ao.get("outbox").unwrap_or_else(|_| {
                let outbox = lua.create_table().unwrap();
                outbox.set("Output", lua.create_table().unwrap()).unwrap();
                outbox.set("Messages", lua.create_table().unwrap()).unwrap();
                outbox.set("Spawns", lua.create_table().unwrap()).unwrap();
                outbox
                    .set("Assignments", lua.create_table().unwrap())
                    .unwrap();
                outbox
            }),
        )?;
    } else {
        ao_lua.set("id", "")?;
        ao_lua.set("_module", "")?;
        ao_lua.set("authorities", lua.create_table()?)?;
        ao_lua.set("reference", 0)?;
        let outbox = lua.create_table()?;
        outbox.set("Output", lua.create_table()?)?;
        outbox.set("Messages", lua.create_table()?)?;
        outbox.set("Spawns", lua.create_table()?)?;
        outbox.set("Assignments", lua.create_table()?)?;
        ao_lua.set("outbox", outbox)?;
    }

    // Set nonExtractableTags
    let non_extractable_tags = lua.create_table()?;
    let net_tags = [
        "Data-Protocol",
        "Variant",
        "From-Process",
        "From-Module",
        "Type",
        "From",
        "Owner",
        "Anchor",
        "Target",
        "Data",
        "Tags",
        "Read-Only",
    ];
    for (i, tag) in net_tags.iter().enumerate() {
        non_extractable_tags.set(i + 1, *tag)?;
    }
    ao_lua.set("nonExtractableTags", non_extractable_tags)?;

    // Set nonForwardableTags
    let non_forwardable_tags = lua.create_table()?;
    let nft_tags = [
        "Data-Protocol",
        "Variant",
        "From-Process",
        "From-Module",
        "Type",
        "From",
        "Owner",
        "Anchor",
        "Target",
        "Tags",
        "TagArray",
        "Hash-Chain",
        "Timestamp",
        "Nonce",
        "Epoch",
        "Signature",
        "Forwarded-By",
        "Pushed-For",
        "Read-Only",
        "Cron",
        "Block-Height",
        "Reference",
        "Id",
        "Reply-To",
    ];
    for (i, tag) in nft_tags.iter().enumerate() {
        non_forwardable_tags.set(i + 1, *tag)?;
    }
    ao_lua.set("nonForwardableTags", non_forwardable_tags)?;

    // Set Nonce to nil
    ao_lua.set("Nonce", LuaValue::Nil)?;

    // Set implemented Rust functions
    ao_lua.set("clone", lua.create_function(clone)?)?;
    ao_lua.set("normalize", lua.create_function(normalize)?)?;
    ao_lua.set("sanitize", lua.create_function(sanitize)?)?;
    //ao_lua.set("init", lua.create_function(init)?)?;
    ao_lua.set("log", lua.create_function(log)?)?;
    //ao_lua.set("clearOutbox", lua.create_function(clearOutbox)?)?;
    //ao_lua.set("send", lua.create_function(send)?)?;
    //ao_lua.set("spawn", lua.create_function(spawm)?)?;
    //ao_lua.set("assign", lua.create_function(assign)?)?;
    //ao_lua.set("isTrusted", lua.create_function(is_trusted)?)?;
    //ao_lua.set("result", lua.create_function(result)?)?;
    Ok(ao_lua)
}

/// Logs a message to the output, replacing the Lua version of ao.log.
/// Takes the `ao` table and a text string, appending the text to `ao.outbox.Output`.
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

fn includes(lua: &Lua, list: LuaTable) -> LuaResult<LuaFunction> {
    let func = move |_lua: &Lua, key: LuaValue| -> LuaResult<bool> {
        for pair in list.pairs::<i32, LuaValue>() {
            let (_, list_key) = pair?;
            if list_key == key {
                return Ok(true);
            }
        }
        Ok(false)
    };
    lua.create_function(func)
}

fn pad_zero32(_lua: &Lua, num: i64) -> LuaResult<String> {
    Ok(format!("{:032}", num))
}

fn clone(lua: &Lua, (obj, seen): (LuaValue, Option<LuaTable>)) -> LuaResult<LuaValue> {
    // Handle non-tables
    if !matches!(obj, LuaValue::Table(_)) {
        return Ok(obj);
    }

    let obj_table = match obj {
        LuaValue::Table(t) => t,
        _ => unreachable!(),
    };

    // Handle seen tables
    let seen = seen.unwrap_or_else(|| lua.create_table().unwrap());
    if let Ok(existing) = seen.get::<LuaValue>(LuaValue::Table(obj_table.clone())) {
        return Ok(existing);
    }

    // Create new table and mark as seen
    let res = lua.create_table()?;
    seen.set(obj_table.clone(), res.clone())?;

    // Copy key-value pairs recursively
    for pair in obj_table.pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair?;
        let cloned_key = clone(lua, (k, Some(seen.clone())))?;
        let cloned_value = clone(lua, (v, Some(seen.clone())))?;
        res.set(cloned_key, cloned_value)?;
    }

    // Copy metatable
    if let Some(mt) = obj_table.metatable() {
        res.set_metatable(Some(mt));
    }

    Ok(LuaValue::Table(res))
}

fn normalize(lua: &Lua, msg: LuaTable) -> LuaResult<LuaTable> {
    let tags: LuaTable = msg.get("Tags")?;
    let non_extractable_tags: LuaTable = lua
        .globals()
        .get::<LuaTable>("ao")?
        .get("nonExtractableTags")?;
    let includes_fn = includes(lua, non_extractable_tags)?;

    for pair in tags.sequence_values::<LuaTable>() {
        let tag = pair?;
        let name: String = tag.get("name")?;
        let includes_result: bool = includes_fn.call(name.clone())?;
        if !includes_result {
            let value: LuaValue = tag.get("value")?;
            msg.set(name, value)?;
        }
    }

    Ok(msg)
}

fn sanitize(lua: &Lua, msg: LuaTable) -> LuaResult<LuaTable> {
    let cloned_value = clone(lua, (LuaValue::Table(msg), None))?;
    let new_msg = match cloned_value {
        LuaValue::Table(t) => t,
        _ => {
            return Err(LuaError::RuntimeError(
                "Clone did not return a table".to_string(),
            ))
        }
    };
    let non_forwardable_tags: LuaTable = lua
        .globals()
        .get::<LuaTable>("ao")?
        .get("nonForwardableTags")?;
    let includes_fn = includes(lua, non_forwardable_tags)?;

    for pair in new_msg.pairs::<LuaValue, LuaValue>() {
        let (key, _) = pair?;
        let includes_result: bool = includes_fn.call(&key)?;
        if includes_result {
            new_msg.set(key, LuaValue::Nil)?;
        }
    }

    Ok(new_msg)
}
