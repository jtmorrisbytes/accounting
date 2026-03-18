fn main() -> Result<(), Box<dyn std::error::Error>> {
    const MANIFEST_ENV_VAR: &str = "CARGO_MANIFEST_DIR";
    let manifest_root = std::env::var(MANIFEST_ENV_VAR)
        .map_err(|e| format!("Environment variable {MANIFEST_ENV_VAR} error {e}"))?;
    let sqlite_src_dir = std::path::Path::new(&manifest_root)
        .join("vendor")
        .join("sqlite-c-3150300");
    if !sqlite_src_dir.exists() {
        return Err(format!("Faileed to find {}: ", sqlite_src_dir.display()).into());
    }
    // let sqlite_src_dir = sqlite_src_dir.display().to_string();

    // build with cc
    cc::Build::new()
        .file(sqlite_src_dir.join("sqlite3.c"))
        .define("SQLITE_THREADSAFE", "0")
        .define("SQLITE_OMIT_LOAD_EXTENSION", None)
        .define("SQLITE_DEFAULT_MEMSTATUS", "0")
        .flag("-mno-stack-arg-probe")
        .flag("-fno-stack-protector")
        .static_crt(true)
        .opt_level(3)
        .compile("sqlite3");

    // let sqlite_src_string = sqlite_src_dir.display().to_string();

    println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rustc-link-search={}",sqlite_src_dir.display());
    // println!("cargo:rustc-link-lib=static={}","jtmb_sqlite");

    Ok(())
}
