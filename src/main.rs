use mlua::Function;
use mlua::Table;
use mlua::prelude::*;

mod colors;
mod parser;
mod svg;

fn debug_print(lua: &Lua, table: LuaTable) -> LuaResult<()> {
    // get globals
    let globals = lua.globals();

    // Convert to string
    match table.raw_len() {
        0 => {
            // Hash table - iterate through key-value pairs
            println!("Table (hash): {{");
            for pair in table.pairs::<LuaValue, LuaValue>() {
                let (key, value) = pair?;
                println!("  {:?} = {:?}", key, value);
            }
            println!("}}");
        }
        _ => {
            // Array-like table
            println!("Table (array): [");
            for i in 1..=table.raw_len() {
                let value: LuaValue = table.raw_get(i)?;
                println!("  [{}] = {:?}", i, value);
            }
            println!("]");
        }
    }

    Ok(())
}

fn main() {
    // Create the lua struct for managing lua state
    let lua = Lua::new();

    // Setup the inputs (Just a URL for now)
    let job = lua.create_table_from([("url", "google.com")]).unwrap();

    let lua_file = include_str!("../hello-world.lua");
    let lua_sig_junk = parser::parse_lua_docs(lua_file);

    // Get the lua module that we're going to be using..
    let source: Table = lua.load(lua_file).eval().unwrap();

    // Call the only method we care about, which returns a table representing the key values found
    let output = source.call_method::<Table>("build_badge", job).unwrap();

    debug_print(&lua, output).unwrap();
}
