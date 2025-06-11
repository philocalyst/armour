use mlua::prelude::*;
use mlua::Function;
use mlua::Table;

mod parser;

fn main() {
    // Create the lua struct for managing lua state
    let lua = Lua::new();

    // Setup the inputs (Just a URL for now)
    let job = lua.create_table_from([("url", "google.com")]).unwrap();

    let lua_file = include_str!("../hello-world.lua");
    let lua_sig_junk = parser::parse_lua_docs(lua_file);

    // Get the lua module that we're going to be using..
    let source: Table = lua.load(lua_file).eval().unwrap();

    // Call the only method we care about
    source.call_method::<()>("build_badge", job).unwrap();
}
