use std::sync::Arc;

// we plan on implementing a vault ui based on https and html
// reexport the rocket crate;
pub use rocket;
use rocket::tokio;
use rocket::{Build, Rocket};
use wasmer::{
    Engine, Function, FunctionEnv, Instance, Module, Store,
    sys::{CpuFeature, Features, NativeEngineExt, Target, Triple},
};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_wasix::{
    PluggableRuntime, WasiEnv, WasiEnvBuilder, generate_import_object_from_env,
    runtime::task_manager::tokio::TokioTaskManager,
};

fn run_wasmer_webassembly(wasm_bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let module = Module::new(&store, &wasm_bytes)?;

    let rt = Arc::new(PluggableRuntime::new(Arc::new(TokioTaskManager::new(
        tokio::runtime::Handle::current(),
    ))));

    let mut wasi_env_builder = WasiEnv::builder("vault-ui");
    wasi_env_builder.set_runtime(rt);
    wasi_env_builder = wasi_env_builder.env("key", "value");
    wasi_env_builder.set_engine(Engine::new(
        Box::new(Cranelift::default()),
        Target::new(Triple::host(), CpuFeature::for_host()),
        Features::detect_from_wasm(wasm_bytes)?,
    ));
    let (i, wasi_env) = wasi_env_builder.instantiate(module, &mut store)?;
    // let mut fn_env = FunctionEnv::new(&mut store, wasi_env);
    // let imports = generate_import_object_from_env(
    //     &mut store,
    //     &mut fn_env,
    //     wasmer_wasix::WasiVersion::Wasix64v1,
    // );
    // we have to register wasi_snapshot_preview functions
    // let snapshot1_exports = wasmer::Exports::new();
    // snapshot1_exports.insert("environ_get", value);
    // imports.register_namespace("wasi_snapshot_preview_1", Function::new_typed_with_env(&mut store, &fn_env, func) );

    // let i = Instance::new(&mut store, &module, &imports)?;
    let start = i.exports.get::<Function>("_start")?;
    start.call(&mut store, &[])?;
    Ok(())
}

// bitches we crazy yo!
// we goin to compile a rust crate as a ui library at dev time so we can SSR render that webasembly yo

// obviously only use this during developmentt

pub fn load_wasm_from_crate_root_devtime(lib_name: &str) -> Vec<u8> {
    // let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();
    // let package = metadata
    //     .packages
    //     .iter()
    //     .find(|p| p.name == "vault_wasm_ui")
    //     .unwrap();
    // let t = package
    //     .targets
    //     .iter()
    //     .find(|t| t.kind.contains(&cargo_metadata::TargetKind::Lib) && t.name == lib_name)
    //     .unwrap();
    // let out_dir = env!("OUT_DIR");
    todo!();
    let mut f =
        std::fs::File::open(std::path::Path::new("").join("libvault_ui_wasm.wasm")).unwrap();

    let mut bin = vec![];
    std::io::copy(&mut f, &mut bin).unwrap();
    bin
}

/// displays hello world to the user
#[rocket::get("/")]
pub fn get_index() -> String {
    let bytes = load_wasm_from_crate_root_devtime("wasm_index");
    match run_wasmer_webassembly(&bytes) {
        Ok(_) => "HELLO FROM VAULT".to_string(),
        Err(e) => e.to_string(),
    }
    // wasmtime::
}

// WE ARE HTML AGAIN LOL.
// creates a custom rocket instance based on the vaults config

pub fn build_rocket(vault_config: vault::config::Config) -> Rocket<Build> {
    let config = rocket::Config::figment()
        .merge(("address", "127.0.0.1"))
        .merge(("port", 8765_u16))
        .merge(("shutdown.ctrlc", false));

    rocket::custom(config)
        .mount("/", rocket::routes![get_index])
        .mount(
            "/public/static",
            rocket::fs::FileServer::from(vault_config.vault_data_directory),
        )
}
