use mlua::Function;
use mlua::Table;
use mlua::prelude::*;
fn main() {
    let lua = Lua::new();
    let job = lua.create_table_from([("url", "google.com")]).unwrap();

    let source: Table = lua.load(include_str!("../hello-world.lua")).eval().unwrap();

    println!("{:?}", source.contains_key("build_badge"));

    source.call_method::<()>("build_badge", job).unwrap();

    println!("Hello, world!");
}
