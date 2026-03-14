use std::fs;
use std::path::PathBuf;

fn main() {
    let scripts_dir = "src/plugins";

    println!("cargo:rerun-if-changed={}", scripts_dir);

    let combined = fs::read_dir(scripts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("scm"))
        .map(|e| {
            println!("cargo:rerun-if-changed={}", e.path().display());
            fs::read_to_string(e.path()).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("plugin.scm");
    fs::write(out_path, combined).unwrap();
}
