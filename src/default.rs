use super::*;

// ANSI color codes matching the assumed Colors table in Lua
const GRAY: &str = "\x1b[90m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

/// Registers the `default` module with Lua, providing a default message handler.
/// This handler formats and prints incoming messages, inserting them into the inbox.
#[mlua::lua_module]
pub fn default(lua: &Lua) -> LuaResult<LuaFunction> {
    let default_fn = move |lua: &Lua, insert_inbox: LuaFunction| {
        let handler = move |lua: &Lua, msg: LuaTable| {
            // Insert the message into the inbox
            insert_inbox.call::<()>(msg.clone())?;

            // Get and format the sender's address
            let from: Option<String> = msg.get("From")?;
            let from_str = if let Some(f) = from {
                let len = f.len();
                let first_three = &f[0..core::cmp::min(3, len)];
                let last_three = if len > 3 { &f[len - 3..] } else { &f };
                format!("{}...{}", first_three, last_three)
            } else {
                "unknown".to_string()
            };

            // Start building the output string
            let mut txt = format!("{}New Message From {}{}{}: ", GRAY, GREEN, from_str, GRAY);

            // Check for an Action field
            let action: Option<String> = msg.get("Action")?;
            if let Some(act) = action {
                let action_display = if act.len() > 20 { &act[0..20] } else { &act };
                txt.push_str(&format!("{}Action = {}{}{}", GRAY, BLUE, action_display, RESET));
            } else {
                // Handle the Data field
                let data: LuaValue = msg.get("Data")?;
                let data_str = match data {
                    LuaValue::Table(t) => {
                        // Safely attempt to serialize table data using json.encode
                        let json: LuaResult<LuaTable> = lua.globals().get("json");
                        match json {
                            Ok(json_table) => {
                                let encode: LuaResult<LuaFunction> = json_table.get("encode");
                                match encode {
                                    Ok(encode_fn) => {
                                        match encode_fn.call::<LuaValue>(t) {
                                            Ok(LuaValue::String(s)) => s.to_str()?.to_string(),
                                            _ => "<unserializable>".to_string(),
                                        }
                                    }
                                    _ => "<json encode unavailable>".to_string(),
                                }
                            }
                            _ => "<json unavailable>".to_string(),
                        }
                    }
                    LuaValue::Nil => "".to_string(),
                    _ => data.to_string()?, // Convert other types directly to string
                };
                let data_display = if data_str.len() > 20 { &data_str[0..20] } else { &data_str };
                txt.push_str(&format!("{}Data = {}{}{}", GRAY, BLUE, data_display, RESET));
            }

            // Print the formatted message
            let print: LuaFunction = lua.globals().get("print")?;
            print.call::<()>(txt)?;

            Ok(())
        };
        lua.create_function(handler)
    };
    lua.create_function(default_fn)
}