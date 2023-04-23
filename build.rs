use std::{ffi::OsStr, path::PathBuf, process::Command};

fn main() {
    // Open Cargo.toml
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = format!("{manifest_dir}/Cargo.toml");
    let manifest_str = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("Could not open {manifest_path}: {e}"));
    let manifest_data: toml::Value =
        toml::de::from_str(&manifest_str).expect("Could not parse Cargo manifest as TOML");

    // Find the romfs setting and compute the path
    let romfs_dir_setting = manifest_data
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("metadata"))
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("cargo-3ds"))
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("romfs_dir"))
        .and_then(toml::Value::as_str)
        .unwrap_or("romfs");
    let mut romfs_path: PathBuf = PathBuf::from(&manifest_dir);
    romfs_path.push(PathBuf::from(romfs_dir_setting));

    // Check if the romfs path exists so we can compile the module
    if romfs_path.exists() {
        println!("cargo:rustc-cfg=romfs_exists");
    }
    println!("cargo:rerun-if-changed={}", &manifest_dir);

    let mut asset_dir = PathBuf::from(&manifest_dir);
    asset_dir.push("src");
    asset_dir.push("assets");

    for entry in asset_dir.read_dir().unwrap().flatten() {
        println!("Checking {:?}", entry.path().display());
        if let Some("pica") = entry.path().extension().and_then(OsStr::to_str) {
            println!("cargo:rerun-if-changed={}", entry.path().display());

            let mut out_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
            out_path.push("src");
            out_path.push("assets");
            out_path.push(entry.path().with_extension("shbin").file_name().unwrap());

            std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

            println!("Compiling {:?}", out_path.display());

            let mut cmd = Command::new("picasso");
            cmd.arg(entry.path()).arg("--out").arg(out_path);

            let status = cmd.spawn().unwrap().wait().unwrap();
            assert!(
                status.success(),
                "Command {cmd:#?} failed with code {status:?}"
            );
        }
    }
}
