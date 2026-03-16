fn main() -> Result<(), Box<dyn std::error::Error>> {
    const MANIFEST_ENV_VAR: &str = "CARGO_MANIFEST_DIR";
    let workspace_root = std::env::var(MANIFEST_ENV_VAR).map_err(|e|format!("Environment variable {MANIFEST_ENV_VAR} error {e}"))?;
    let zig_out = std::path::Path::new(&workspace_root).join("..").join("zig-out").join("lib");
    let zig_out = zig_out.canonicalize().map_err(|e|format!("Faileed to find {}: {e}",zig_out.display()))?;
    let zig_out = zig_out.display().to_string();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-search={}",zig_out);
    println!("cargo:rustc-link-lib=static={}","jtmb_sqlite");

    Ok(())
}