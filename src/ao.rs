
use super::*;

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
    ao_lua.set("log", lua.create_function(log)?)?;

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
