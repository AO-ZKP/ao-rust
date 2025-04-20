use super::*;

// Eval module initialization
#[mlua::lua_module(name = "eval")]
pub fn eval_module(lua: &Lua) -> LuaResult<LuaFunction> {
    // Create the outer function that takes `ao` and returns the handler
    lua.create_function(|lua, ao: LuaTable| {
        // Create the inner handler function that captures `ao`
        let handler = lua.create_function(move |lua, msg: LuaTable| {
            // Extract the expression from msg.Data
            let expr: String = msg.get("Data")?;

            // Try loading with "return " prefix, fallback to direct expression
            let func = match lua.load(&format!("return {}", expr)).into_function() {
                Ok(f) => f,
                Err(_) => lua.load(expr).into_function()?,
            };

            // Create a Lua thread (coroutine) to execute the function
            let thread = lua.create_thread(func)?;

            // Resume the thread with no arguments
            match thread.resume::<Option<LuaValue>>(()) {
                Ok(Some(value)) => {
                    // Execution completed successfully, handle the output
                    handle_output(lua, &ao, value)?;
                }
                Ok(None) => {
                    // Thread yielded, let AoLoader handle the coroutine state
                    // No action needed here; the thread state is preserved in memory
                }
                Err(e) => {
                    // Execution failed, set the error
                    let outbox: LuaTable = ao.get("outbox")?;
                    let error_msg = match e {
                        mlua::Error::RuntimeError(msg) => msg,
                        _ => e.to_string(),
                    };
                    // Set outbox.Error
                    outbox.set("Error", error_msg.clone())?;
                    // Set global Errors variable to store the last error
                    lua.globals().set("Errors", error_msg)?;
                }
            }

            Ok(())
        })?;
        Ok(handler)
    })
}

// Helper to handle successful output
fn handle_output(lua: &Lua, ao: &LuaTable, output: LuaValue) -> LuaResult<()> {
    // Check if HANDLER_PRINT_LOGS is set
    let handler_print_logs: Option<LuaTable> = lua.globals().get("HANDLER_PRINT_LOGS")?;
    if let Some(logs) = handler_print_logs {
        if !matches!(output, LuaValue::Nil) {
            let formatted = format_value(lua, output)?;
            logs.raw_set(logs.raw_len() + 1, formatted)?;
        }
    } else {
        // Set ao.outbox.Output with json, data, and prompt
        let outbox: LuaTable = ao.get("outbox")?;
        let output_table = lua.create_table()?;

        // Set json field
        let json_value = match output {
            LuaValue::Table(ref t) => {
                let stringify = require_module(lua, "stringify")?;
                let format_fn: LuaFunction = stringify.get("format")?;
                let formatted: String = format_fn.call((t.clone(), None::<i32>, None::<LuaTable>))?;
                LuaValue::String(lua.create_string(&formatted)?)
            }
            _ => LuaValue::String(lua.create_string("undefined")?),
        };
        output_table.set("json", json_value)?;

        // Set data table
        let data_table = lua.create_table()?;
        let data_output = format_value(lua, output.clone())?;
        data_table.set("output", data_output)?;
        data_table.set("prompt", prompt(lua)?)?;
        output_table.set("data", data_table)?;

        // Set prompt at top level
        output_table.set("prompt", prompt(lua)?)?;

        outbox.set("Output", output_table)?;
    }
    Ok(())
}

// Helper to format a value
fn format_value(lua: &Lua, value: LuaValue) -> LuaResult<LuaValue> {
    match value {
        LuaValue::Table(t) => {
            let stringify = require_module(lua, "stringify")?;
            let format_fn: LuaFunction = stringify.get("format")?;
            let formatted: String = format_fn.call((t, None::<i32>, None::<LuaTable>))?;
            Ok(LuaValue::String(lua.create_string(&formatted)?))
        }
        other => Ok(other),
    }
}

// Helper to require a module
fn require_module(lua: &Lua, name: &str) -> LuaResult<LuaTable> {
    let require: LuaFunction = lua.globals().get("require")?;
    require.call(name)
}

// Helper to call the global Prompt function
fn prompt(lua: &Lua) -> LuaResult<LuaValue> {
    let prompt_fn: LuaFunction = lua.globals().get("Prompt")?;
    prompt_fn.call(())
}