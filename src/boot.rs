use alloc::format;
use alloc::string::{String, ToString};
use mlua::prelude::*;

/// Reads data from a file in the '/data/' directory using Lua's io library.
/// Returns `Some(contents)` if successful, `None` if the file is not found,
/// or an error if the operation fails unexpectedly.
fn get_data(lua: &Lua, tx_id: &str) -> LuaResult<Option<String>> {
    let io: LuaTable = lua.globals().get("io")?;
    let filename = format!("/data/{}", tx_id);
    let open: LuaFunction = io.get("open")?;
    let file_result: LuaValue = open.call((filename, "r"))?;
    match file_result {
        LuaValue::UserData(file) => {
            let contents: String = file.call_method("read", "*a")?;
            file.call_method::<()>("close", ())?; // Specify return type as ()
            Ok(Some(contents))
        }
        LuaValue::Nil => Ok(None),
        _ => Err(LuaError::RuntimeError("io.open returned unexpected value".to_string())),
    }
}

/// The boot module function, registered with Lua via the `mlua` crate.
#[mlua::lua_module]
pub fn boot(lua: &Lua) -> LuaResult<LuaFunction> {
    let require: LuaFunction = lua.globals().get("require")?;
    let eval_module: LuaFunction = require.call("eval")?;
    let boot_fn = move |lua: &Lua, ao: LuaTable| {
        let eval_handler: LuaFunction = eval_module.call(ao.clone())?;
        let handler = move |lua: &Lua, msg: LuaTable| {
            let inbox: LuaTable = lua.globals().get("Inbox")?;
            if inbox.len()? == 0 {
                inbox.push(msg.clone())?;
            }
            let tags: LuaTable = msg.get("Tags")?;
            let on_boot: Option<String> = tags.get("On-Boot")?;
            if let Some(on_boot_value) = on_boot {
                if on_boot_value == "Data" {
                    eval_handler.call::<()>(msg.clone())?; // Specify return type as ()
                } else {
                    let loaded_val = get_data(lua, &on_boot_value)?;
                    if let Some(data) = loaded_val {
                        let eval_msg = lua.create_table()?;
                        eval_msg.set("Data", data)?;
                        eval_handler.call::<()>(eval_msg)?; // Specify return type as ()
                    }
                }
            }
            Ok(())
        };
        lua.create_function(handler)
    };
    lua.create_function(boot_fn)
}