fn main() -> Result<(), &'static str> {
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR").ok_or("environment variable CARGO_MANIFEST_DIR not set")?;
    println!("cargo:rerun-if-changed=build.rs");
    prinln!("cargo:rustc-link-search")
}