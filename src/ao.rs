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
    ao_lua.set("init", lua.create_function(init)?)?;
    ao_lua.set("log", lua.create_function(log)?)?;
    ao_lua.set("clearOutbox", lua.create_function(clear_outbox)?)?;
    ao_lua.set("send", lua.create_function(send)?)?;
    //ao_lua.set("spawn", lua.create_function(spawn)?)?;
    ao_lua.set("assign", lua.create_function(assign)?)?;
    ao_lua.set("isTrusted", lua.create_function(is_trusted)?)?;
    ao_lua.set("result", lua.create_function(result)?)?;
    Ok(ao_lua)
}

fn init(lua: &Lua, env: LuaTable) -> LuaResult<()> {
    let ao: LuaTable = lua.globals().get("ao")?;

    // Set ao.id if empty
    let current_id: String = ao.get("id").unwrap_or_default();
    if current_id.is_empty() {
        let process: LuaTable = env.get("Process")?;
        let new_id: String = process.get("Id")?;
        ao.set("id", new_id)?;
    }

    // Set ao._module from Process.Tags
    let current_module: String = ao.get("_module").unwrap_or_default();
    if current_module.is_empty() {
        let process: LuaTable = env.get("Process")?;
        let tags: LuaTable = process.get("Tags")?;
        
        for pair in tags.pairs::<i32, LuaTable>() {
            let (_, tag) = pair?;
            let name: String = tag.get("name")?;
            if name == "Module" {
                let value: String = tag.get("value")?;
                ao.set("_module", value)?;
                break;
            }
        }
    }

    // Populate authorities if empty
    let authorities: LuaTable = ao.get("authorities")?;
    if authorities.len()? == 0 {
        let process: LuaTable = env.get("Process")?;
        let tags: LuaTable = process.get("Tags")?;
        let new_authorities = lua.create_table()?;

        for pair in tags.pairs::<i32, LuaTable>() {
            let (_, tag) = pair?;
            let name: String = tag.get("name")?;
            if name == "Authority" {
                let value: String = tag.get("value")?;
                new_authorities.push(value)?;
            }
        }
        ao.set("authorities", new_authorities)?;
    }

    // Initialize outbox with pure Lua tables
    let outbox = lua.create_table()?;
    outbox.set("Output", lua.create_table()?)?;
    outbox.set("Messages", lua.create_table()?)?;
    outbox.set("Spawns", lua.create_table()?)?;
    outbox.set("Assignments", lua.create_table()?)?;
    ao.set("outbox", outbox)?;

    // Store environment reference
    ao.set("env", env)?;

    Ok(())
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

// Single unified function with embedded wrapper
fn clear_outbox(lua: &Lua, _: ()) -> LuaResult<()> {
    let ao: LuaTable = lua.globals().get("ao")?;
    
    // Create fresh outbox structure
    let outbox = lua.create_table()?;
    outbox.set("Output", lua.create_table()?)?;
    outbox.set("Messages", lua.create_table()?)?;
    outbox.set("Spawns", lua.create_table()?)?;
    outbox.set("Assignments", lua.create_table()?)?;
    
    ao.set("outbox", outbox)?;
    Ok(())
}

fn send(lua: &Lua, msg: LuaTable) -> LuaResult<LuaTable> {
    let ao: LuaTable = lua.globals().get("ao")?;

    // Assert msg is a table
    let lua_assert: LuaFunction = lua.globals().get("assert")?;
    let type_fn: LuaFunction = lua.globals().get("type")?;
    let msg_type: String = type_fn.call::<String>(msg.clone())?;
    lua_assert.call::<()>((msg_type == "table", "msg should be a table"))?;

    // Increment reference
    let mut reference: i64 = ao.get("reference")?;
    reference += 1;
    ao.set("reference", reference)?;
    let reference_str = reference.to_string();

    // Create base message
    let message = lua.create_table()?;
    message.set("Target", msg.get::<String>("Target")?)?;
    message.set("Data", msg.get::<LuaValue>("Data")?)?;
    message.set("Anchor", pad_zero32(reference)?)?;

    // Initialize Tags table
    let tags = lua.create_table()?;
    let mut tag_index = 1;

    // Add default tags
    tags.set(tag_index, create_tag(lua, "Data-Protocol", "ao")?)?;
    tag_index += 1;
    tags.set(tag_index, create_tag(lua, "Variant", "ao.TN.1")?)?;
    tag_index += 1;
    tags.set(tag_index, create_tag(lua, "Type", "Message")?)?;
    tag_index += 1;
    tags.set(tag_index, create_tag(lua, "Reference", &reference_str)?)?;
    tag_index += 1;

    // Add custom tags from msg root
    for pair_result in msg.pairs::<LuaValue, LuaValue>() {
        let (key_value, val_value) = pair_result?;
        let key = key_value.to_string()?;
        if !["Target", "Data", "Anchor", "Tags", "From"].contains(&&*key) {
            let value = val_value.to_string()?;
            tags.set(tag_index, create_tag(lua, &key, &value)?)?;
            tag_index += 1;
        }
    }

    // Handle msg.Tags
    if let Ok(msg_tags) = msg.get::<LuaTable>("Tags") {
        if is_array(lua, LuaValue::Table(msg_tags.clone()))? {
            for pair in msg_tags.sequence_values::<LuaTable>() {
                let o = pair?;
                tags.set(tag_index, o)?;
                tag_index += 1;
            }
        } else {
            for pair_result in msg_tags.pairs::<LuaValue, LuaValue>() {
                let (k, v) = pair_result?;
                let tag = create_tag(lua, &k.to_string()?, &v.to_string()?)?;
                tags.set(tag_index, tag)?;
                tag_index += 1;
            }
        }
    }
    message.set("Tags", tags)?;

    // Early return if Handlers is not present
    if lua.globals().get::<LuaTable>("Handlers").is_err() {
        return Ok(message);
    }

    // Clone message for outbox
    let ext_message = lua.create_table()?;
    for pair_result in message.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair_result?;
        ext_message.set(key, value)?;
    }

    // Add to ao.outbox.Messages
    let outbox: LuaTable = ao.get("outbox")?;
    let messages: LuaTable = outbox.get("Messages")?;
    let current_length = messages.len()?;
    messages.set(current_length + 1, ext_message)?;

    // Add onReply function
    let message_clone = message.clone();
    let reference_str_clone = reference_str.clone();
    message.set("onReply", lua.create_function(move |lua, args: LuaMultiValue| {
        let handlers: LuaTable = lua.globals().get("Handlers")?;
        let (from, resolver) = match args.len() {
            2 => (args[0].clone(), args[1].clone()),
            _ => (message_clone.get::<String>("Target")?.into_lua(lua)?, args[0].clone()),
        };
        let params = lua.create_table()?;
        params.set("From", from)?;
        params.set("X-Reference", &*reference_str_clone)?;
        handlers.get::<LuaFunction>("once")?.call::<()>((params, resolver))?;
        Ok(())
    })?)?;

    // Add receive function
    let message_clone = message.clone();
    let reference_str_clone = reference_str.clone();
    message.set("receive", lua.create_function(move |lua, args: LuaMultiValue| -> LuaResult<LuaMultiValue> {
        let handlers: LuaTable = lua.globals().get("Handlers")?;
        let from = match args.len() {
            1 => args[0].clone(),
            _ => message_clone.get::<String>("Target")?.into_lua(lua)?,
        };
        let params = lua.create_table()?;
        params.set("From", from)?;
        params.set("X-Reference", &*reference_str_clone)?;
        handlers.get::<LuaFunction>("receive")?.call(params)
    })?)?;

    Ok(message)
}

// Helper function to create tag tables
fn create_tag(lua: &Lua, name: &str, value: &str) -> LuaResult<LuaTable> {
    let tag = lua.create_table()?;
    tag.set("name", name)?;
    tag.set("value", value)?;
    Ok(tag)
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

fn pad_zero32(num: i64) -> LuaResult<String> {
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

fn assign(lua: &Lua, assignment: LuaTable) -> LuaResult<()> {
    // Get the global assert function
    let lua_assert: LuaFunction = lua.globals().get("assert")?;
    let type_fn: LuaFunction = lua.globals().get("type")?;
    
    // Check that assignment is a table
    let assignment_type: String = type_fn.call(assignment.clone())?;
    lua_assert.call::<()>((assignment_type == "table", "assignment should be a table"))?;
    
    // Check that assignment.Processes is a table
    let processes: LuaValue = assignment.get("Processes")?;
    let processes_type: String = type_fn.call(processes)?;
    lua_assert.call::<()>((processes_type == "table", "Processes should be a table"))?;
    
    // Check that assignment.Message is a string
    let message: LuaValue = assignment.get("Message")?;
    let message_type: String = type_fn.call(message)?;
    lua_assert.call::<()>((message_type == "string", "Message should be a string"))?;
    
    // Get the ao table and its outbox
    let ao: LuaTable = lua.globals().get("ao")?;
    let outbox: LuaTable = ao.get("outbox")?;
    let assignments: LuaTable = outbox.get("Assignments")?;
    
    // Insert the new assignment
    let new_index = assignments.len()? + 1;
    assignments.set(new_index, assignment)?;
    
    Ok(())
}

fn is_trusted(lua: &Lua, msg: LuaTable) -> LuaResult<bool> {
    let ao: LuaTable = lua.globals().get("ao")?;
    let authorities: LuaTable = ao.get("authorities")?;
    
    let from: Option<String> = msg.get("From")?;
    let owner: Option<String> = msg.get("Owner")?;
    
    for i in 1..=authorities.len()? {
        let authority: String = authorities.get(i)?;
        if from.as_ref() == Some(&authority) || owner.as_ref() == Some(&authority) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

fn result(lua: &Lua, res: LuaTable) -> LuaResult<LuaTable> {
    let ao: LuaTable = lua.globals().get("ao")?;
    let outbox: LuaTable = ao.get("outbox")?;
    
    // Check for errors first
    let error_msg: Option<String> = res.get("Error")?;
    let outbox_error: Option<String> = outbox.get("Error")?;
    
    if error_msg.is_some() || outbox_error.is_some() {
        let error_table = lua.create_table()?;
        error_table.set("Error", error_msg.or(outbox_error))?;
        return Ok(error_table);
    }
    
    // No errors, return full result
    let result_table = lua.create_table()?;
    result_table.set("Output", res.get::<Option<LuaValue>>("Output")?.unwrap_or_else(|| outbox.get("Output").unwrap_or(LuaValue::Nil)))?;
    result_table.set("Messages", outbox.get::<LuaTable>("Messages")?)?;
    result_table.set("Spawns", outbox.get::<LuaTable>("Spawns")?)?;
    result_table.set("Assignments", outbox.get::<LuaTable>("Assignments")?)?;
    
    Ok(result_table)
}