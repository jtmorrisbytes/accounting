use std::{collections::HashMap, str::FromStr};

use cargo_metadata::camino::Utf8PathBuf;
use flate2::read::GzDecoder;
// use tokio::io::AsyncBufReadExt;

// bundle the service and its dependencies to prepare it for installation or distrobution
const X86_64_PC_WINDOWS_MSVC: &'static str = "x86_64-pc-windows-msvc";
const TARGET_I686_PC_WINDOWS_MSVC: &str = "i686-pc-windows-msvc";
/// wasm32 wasi preview 1
const TARGET_WASM32_WASIP1: &str = "wasm32-wasip1";
const TARGET_WASM32_UNKNOWN_UNKNOWN: &str = "wasm32-unknown-unknown";
const TARGET_X86_64_UNKNOWN_LINUX_GNU: &'static str = "x86_64-unknown-linux-gnu";

const RELEASE_TYPE_RELEASE: &'static str = "release";
const WASM_FILE_EXTENSION: &str = ".wasm";

const VAULT_UI_PKG_NAME: &str = "vault_wasm_ui";
// const VAULT_UI_WASM_NAME: &str = concat!(VAULT_UI_PKG_NAME,WASM_FILE_EXTENTION);

fn vault_ui_pkg_wasm_name() -> String {
    format!("{VAULT_UI_PKG_NAME}{WASM_FILE_EXTENSION}")
}

fn pkg_name_to_wasm_fiename(name: &cargo_metadata::PackageName<String>) -> String {
    name.to_string() + WASM_FILE_EXTENSION
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ZigArchEntry {
    version: Option<String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct ZigMasterEntry {}

#[derive(serde::Serialize, serde::Deserialize)]
struct ZigIndexEntrySrc {
    tarball: String,
    shasum: String,
    size: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ZigIndexEntry {
    version: Option<String>,
    date: String,
    docs: String,
    #[serde(rename = "stdDocs")]
    std_docs: Option<String>,
    src: ZigIndexEntrySrc,
    bootstrap: Option<ZigIndexEntrySrc>,
    #[serde(rename = "x86_64-macos")]
    x86_64_macos: Option<ZigIndexEntrySrc>,

    #[serde(rename = "aarch64-macos")]
    aarch64_macos: Option<ZigIndexEntrySrc>,

    #[serde(rename = "x86_64-linux")]
    x86_64_linux: Option<ZigIndexEntrySrc>,

    #[serde(rename = "aarch64-linux")]
    aarch64_linux: Option<ZigIndexEntrySrc>,

    #[serde(rename = "arm-linux")]
    arm_linux: Option<ZigIndexEntrySrc>,

    #[serde(rename = "riscv64-linux")]
    riscv64_linux: Option<ZigIndexEntrySrc>,

    #[serde(rename = "powerpc64le-linux")]
    powerpc64le_linux: Option<ZigIndexEntrySrc>,

    #[serde(rename = "x86-linux")]
    x86_linux: Option<ZigIndexEntrySrc>,

    #[serde(rename = "loongarch64-linux")]
    longarch64_linux: Option<ZigIndexEntrySrc>,

    #[serde(rename = "s390x-linux")]
    s39x_linux: Option<ZigIndexEntrySrc>,

    #[serde(rename = "x86_64-windows")]
    x86_64_windows: ZigIndexEntrySrc,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ZigIndex(HashMap<String, ZigIndexEntry>);

async fn try_dowload_zig(workspace_root: Utf8PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::io::AsyncBufReadExt;
    // download the community mirror
    let txt_response = reqwest::get("https://ziglang.org/download/community-mirrors.txt").await?;
    let status = txt_response.status();
    if !status.is_success() {
        return Err(format!(
            "Request to get zigbuild community mirror list failed with status {status}"
        )
        .into());
    }
    let raw_body = txt_response.bytes().await?;
    let mut c = std::io::Cursor::new(raw_body);

    loop {
        let mut line = String::new();
        let r = c.read_line(&mut line).await;
        match r {
            Ok(n) if n > 0 => {}
            Ok(_) => {
                // end of the list
                return Err("Failed to download zig because we have exhausted all the community mirrors available (no more data left)".into());
            }
            Err(e) => {
                return Err(e.into_inner().unwrap());
            }
        }
        // make a request to the mirror
        println!("{}", line);
        // download the package index to get the information
        let index_json_response = reqwest::get(format!("{line}/index.json")).await?;
        if !index_json_response.status().is_success() {
            continue;
        }
        // if index_json_response.content_length().unwrap_or(0) == 0 {
        //     continue;
        // };

        let index = match index_json_response.json::<ZigIndex>().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        let version_info = index.0.get("0.15.2");
        if version_info.is_none() {
            continue;
        }
        let version_info = version_info.unwrap();
        dbg!(&version_info.x86_64_windows.tarball);
        let tarball_url = reqwest::Url::from_str(&version_info.x86_64_windows.tarball)?;
        let tarball_request = match reqwest::get(tarball_url.clone()).await {
            Ok(tarball) => tarball,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        let _ = tokio::fs::create_dir_all(workspace_root.join("zig"))
            .await
            .ok();
        let filename: String = tarball_url
            .path_segments()
            .into_iter()
            .last()
            .unwrap()
            .collect();

        let mut cursor = std::io::Cursor::new(tarball_request.bytes().await?);

        if filename.ends_with(".zip") {
            let mut a = zip::ZipArchive::new(cursor)?;
            a.extract(workspace_root.join("zig"))?;
        } else if filename.ends_with(".tar.xz") {
            let gz_stream = xz2::read::XzDecoder::new(cursor);
            let reader = std::io::BufReader::new(gz_stream);
            let mut archive = tar::Archive::new(reader);
            archive.unpack(workspace_root.join("zig"))?;
        }
        break;
    }
    Ok(())
}

fn install_required_targets_using_rustup(
    targets: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let status = std::process::Command::new("rustup")
        .args(&["target", "add"])
        .args(targets)
        .status()?;
    if !status.success() {
        return Err("Failed to install required targets".into());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // todo. install rust toolchain and compile subscripts and dependencies
    println!(
        "This command will compile the local source code and prepare it for installation or distrobution"
    );
    install_required_targets_using_rustup(&[
        TARGET_I686_PC_WINDOWS_MSVC,
        TARGET_I686_PC_WINDOWS_MSVC,
        TARGET_WASM32_UNKNOWN_UNKNOWN,
        TARGET_WASM32_WASIP1,
        TARGET_X86_64_UNKNOWN_LINUX_GNU,
    ])?;

    let metadata = cargo_metadata::MetadataCommand::new().exec()?;
    // check if zig in project root. assume zig in worskpace zig
    if !std::fs::exists(metadata.workspace_root.join("zig")).unwrap_or(false) {
        println!("Zig required for build. downloading zig");
        try_dowload_zig(metadata.workspace_root.clone()).await?;
    }

    // the name of the package 'this software'
    const PACKAGE_NAME: &str = "vault_service";
    let package_metadata = metadata
        .packages
        .iter()
        .find(|p| p.name == PACKAGE_NAME)
        .ok_or("Expected to find the package in the current directory tree")?;
    // package_metadata.manifest_path;

    // std::process::Command::new("rustup target add x86_64-pc-windows-msvc")
    #[cfg(target_os = "windows")]
    {
        println!("running tests");

        let s = std::process::Command::new("cargo")
            .args(&[
                "test",
                "--target",
                X86_64_PC_WINDOWS_MSVC,
                "--target",
                TARGET_I686_PC_WINDOWS_MSVC,
                "--release",
                "--manifest-path",
                package_metadata.manifest_path.as_str(),
                "-p",
                package_metadata.name.as_str(),
            ])
            .status()?
            .success();
        if !s {
            return Err(format!("Test failed").into());
        }
        std::process::Command::new("cargo")
            .args(&[
                "build",
                "--target",
                X86_64_PC_WINDOWS_MSVC,
                "--target",
                TARGET_I686_PC_WINDOWS_MSVC,
                "--release",
                "--manifest-path",
                package_metadata.manifest_path.as_str(),
            ])
            .status()
            .ok();
    }

    let artifact_dir = metadata.workspace_root.join("out");
    std::fs::remove_dir_all(&artifact_dir).ok();
    std::fs::create_dir_all(&artifact_dir).ok();
    // build the ui wasm for the cranelift compiler and wasmer

    let s = std::process::Command::new("cargo")
        .args(&[
            "build",
            "--lib",
            "--release",
            "--target",
            TARGET_WASM32_WASIP1,
            "--target",
            TARGET_WASM32_UNKNOWN_UNKNOWN,
            "-p",
            VAULT_UI_PKG_NAME,
        ])
        .status()?;
    if !s.success() {
        return Err("Failed to compile webassembly".into());
    }
    // optimize the web assembly before passing it to the cranelift compiler
    let optimization_options = wasm_opt::OptimizationOptions::new_opt_level_4();
    let vault_ui_wasm_filename = vault_ui_pkg_wasm_name();
    optimization_options.run(
        metadata
            .target_directory
            .join(TARGET_WASM32_WASIP1)
            .join(RELEASE_TYPE_RELEASE)
            .join(&vault_ui_wasm_filename),
        artifact_dir.join(&vault_ui_wasm_filename),
    )?;

    // target\wasm32-unknown-unknown\release\vault_wasm_ui.wasm

    optimization_options.run(
        metadata
            .target_directory
            .join(TARGET_WASM32_UNKNOWN_UNKNOWN)
            .join(RELEASE_TYPE_RELEASE)
            .join(vault_ui_wasm_filename),
        artifact_dir.join(format!("{VAULT_UI_PKG_NAME}_wasm32{WASM_FILE_EXTENSION}")),
    )?;

    // copy the windows 64 bit executable

    let exe_name = format!("{}.exe", package_metadata.name);
    std::fs::copy(
        metadata
            .target_directory
            .join(X86_64_PC_WINDOWS_MSVC)
            .join(RELEASE_TYPE_RELEASE)
            .join(&exe_name),
        artifact_dir.join(format!(
            "{X86_64_PC_WINDOWS_MSVC}_{RELEASE_TYPE_RELEASE}_{}.exe",
            package_metadata.name
        )),
    )?;
    std::fs::copy(
        metadata.target_directory.join(
            metadata
                .target_directory
                .join(TARGET_I686_PC_WINDOWS_MSVC)
                .join(RELEASE_TYPE_RELEASE)
                .join(exe_name),
        ),
        artifact_dir.join(format!(
            "{TARGET_I686_PC_WINDOWS_MSVC}_{RELEASE_TYPE_RELEASE}_{}.exe",
            package_metadata.name
        )),
    )?;

    // build the linux service. (may not work right now so these can 'silently' fail)
    build_and_run_tests_linux(&metadata, package_metadata)?;

    Ok(())
}

fn build_and_run_tests_linux(
    metadata: &cargo_metadata::Metadata,
    package_metadata: &cargo_metadata::Package,
) -> Result<(), Box<dyn std::error::Error>> {
    let zigpath = &metadata
        .workspace_root
        .join("zig")
        .join("zig-x86_64-windows-0.15.2");
    dbg!(zigpath);
    let output = std::process::Command::new("cargo")
        .args(&[
            "zigbuild",
            "--tests",
            // "-vv",
            "-p",
            &package_metadata.name,
            &format!("--{RELEASE_TYPE_RELEASE}"),
            "--target",
            TARGET_X86_64_UNKNOWN_LINUX_GNU,
            "--message-format=json",
        ])
        .env("ZIG", &zigpath.canonicalize()?)
        .env(
            "ZIG_EXE_PATH",
            &zigpath.join("zig.exe").canonicalize_utf8()?,
        )
        .env(
            "PATH",
            std::env::var("PATH").unwrap() + ";" + zigpath.as_str(),
        )
        .env("NUM_JOBS", "10")
        .output()?;
    if !output.status.success() {
        return Err("Cargo zigbuild failed for linux".into());
    }
    let mut artifacts = vec![];
    use std::io::BufRead;
    let cursor = std::io::BufReader::new(std::io::Cursor::new(output.stdout));
    for line in cursor.lines() {
        let line = line?;

        let message: serde_json::Value = serde_json::from_str(&line)?;
        if message.get("$message_type")
            == Some(&serde_json::Value::String("diagnostic".to_string()))
        {
            let message_txt = message.get("message").unwrap_or_default();
            let rendered_text = message_txt
                .get("rendered")
                .map(|v| v.to_string())
                .unwrap_or(String::new());
            print!("{rendered_text}");
        }
        if message.get("reason")
            == Some(&serde_json::Value::String("compiler-artifact".to_string()))
            && message.get("executable").is_some()
        {
            let excutable = message.get("executable").unwrap().to_string();
            if excutable != "null" {
                // println!("test artifact? {}", message.get("executable").unwrap());
                artifacts.push(excutable);
            }
        }
    }
    // run each build artifact. should be the test
    for mut artifact in artifacts {
        #[cfg(target_os = "linux")]
        {}
        #[cfg(all(not(target_os = "linux"), target_os = "windows"))]
        {
            // let mut artifact = artifact.replace("\\\\", "/");
            // the start end of the drive letter
            // let drive_path_end = artifact.rfind("\\\\").unwrap();

            // let drive_path = &artifact[0..drive_path_end];
            artifact = artifact.replace("\"", "");
            let drive_letter = artifact.chars().nth(0).unwrap().to_lowercase().to_string();
            // let drive_letter = drive_letter.to_lowercase();
            let artifact = artifact.replace("\\\\", "/");
            let artifact = &artifact[3..];
            // let artifact2 = artifact.replace(drive_path, &drive_letter);
            // these are no longer valid
            // artifact = artifact2;
            let mut artifact = "/mnt/".to_string() + &drive_letter + &artifact;
            artifact = artifact.trim().to_string();
            let wsl = std::process::Command::new("wsl").arg(artifact).status()?;
            if !wsl.success() {
                return Err("Tests failed!".into());
            }
        }
    }
    Ok(())
}
