use mlua::Function;
use mlua::Table;
use mlua::prelude::*;
fn main() {
    // Create the lua struct for managing lua state
    let lua = Lua::new();

    // Setup the inputs (Just a URL for now)
    let job = lua.create_table_from([("url", "google.com")]).unwrap();

    // Get the lua module that we're going to be using..
    let source: Table = lua.load(include_str!("../hello-world.lua")).eval().unwrap();

    // Call the only method we care about
    source.call_method::<()>("build_badge", job).unwrap();
}
