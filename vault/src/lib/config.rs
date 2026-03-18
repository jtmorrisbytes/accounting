// this module to help manage configuration for this program.
// the plan is to run this program as a service

use std::{io::Write, path::PathBuf};

use figment::providers::Format;

/// the program name is based on the name of the package in cargo.toml
const PROGRAM_NAME: &str = env!(
    "CARGO_PKG_NAME",
    "Config module needs the program name to manage configuration"
);
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const CONFIG_FILENAME: &str = "config.toml";

fn fmt_program_name_ascii() -> String {
    PROGRAM_NAME.replace("-", "_").to_ascii_lowercase()
}
fn str_to_wide<S: AsRef<str>>(s: S) -> Vec<u16> {
    s.as_ref().encode_utf16().chain(Some(0)).collect()
}
fn program_name_utf16() -> Vec<u16> {
    let program_name = fmt_program_name_ascii();
    let program_wide = str_to_wide(program_name);
    program_wide
}
pub fn vendor_name() -> String {
    // gets the first author in authors
    let mut s = AUTHORS.split(":");
    let s = s.next().unwrap_or(AUTHORS);
    s.to_string()
}

#[cfg(windows)]
fn win32_open_registry_key<S: AsRef<str>>(
    key: windows::Win32::System::Registry::HKEY,
    sam: windows::Win32::System::Registry::REG_SAM_FLAGS,
    subkey: S,
) -> windows::core::Result<Option<windows::Win32::System::Registry::HKEY>> {
    use windows::Win32::Foundation::{E_INVALIDARG, ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows::Win32::System::Registry::{HKEY, RegOpenKeyExW};
    use windows::core::{HRESULT, PCWSTR};

    if key.is_invalid() {
        return Err(windows::core::Error::new(
            E_INVALIDARG,
            "The program attempted to open a windows registry key with an invalid handle",
        ));
    }

    let subkey_wide = str_to_wide(subkey);
    let subkey = PCWSTR::from_raw(subkey_wide.as_ptr());

    let mut h_subkey = HKEY::default();
    let result = unsafe { RegOpenKeyExW(key, subkey, None, sam, &mut h_subkey) };
    match result {
        ERROR_SUCCESS => Ok(Some(h_subkey)),
        ERROR_FILE_NOT_FOUND => Ok(None),
        other @ _ => {
            return Err(windows::core::Error::from_hresult(HRESULT::from_win32(
                other.0,
            )));
        }
    }
}
#[cfg(target_os = "windows")]
fn win32_get_system_programdata_folder_path() -> windows::core::Result<PathBuf> {
    use windows::Win32::UI::Shell::{
        FOLDERID_ProgramData, KNOWN_FOLDER_FLAG, SHGetKnownFolderPath,
    };

    let pwstr = unsafe {
        windows::Win32::UI::Shell::SHGetKnownFolderPath(
            &FOLDERID_ProgramData,
            KNOWN_FOLDER_FLAG::default(),
            None,
        )?
    };
    let s = unsafe { pwstr.to_string()? };
    Ok(std::path::Path::new(&s).to_path_buf())
}

/// gets the system or local machine data directory. /var/lib/{vendor}/{app} on linux, %PROGRAMDATA%/{vendor}/{app} on windows
pub fn get_service_data_directory() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        Ok(std::path::Path::new("/var/lib")
            .join(vendor_name())
            .join(fmt_program_name_ascii())
            .to_path_buf())
    }
    #[cfg(all(not(target_os = "linux"), target_os = "windows"))]
    {
        win32_get_system_programdata_folder_path()
            .map(|d| d.join(vendor_name()).join(fmt_program_name_ascii()))
            .map_err(|e| e.into())
    }
    #[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
    {
        compile_error!("Unsupported operating system")
    }
}
/// creates the local or system data directory. see get_service_data_directory for details
pub fn create_service_data_directory() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(get_service_data_directory()?)?;
    Ok(())
}

/// gets the local or system configuration directory. /etc/{vendor}/{program} on linux and %PROGRAMDATA%\\{vendor}\\
pub fn get_service_config_directory() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let config_base_path: std::path::PathBuf = {
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/etc")
                .join(vendor_name())
                .join(fmt_program_name_ascii())
                .to_path_buf()
        }
        #[cfg(all(not(target_os = "linux"), target_os = "windows"))]
        {
            // attemt to read the local machines key for this program
            // use windows::Win32

            // let registry_path = format!("Software\\{vendor}\\{program_name}}\\ConfigPath");
            // win32_open_registry_key(windows::Win32::System::Registry::HKEY_LOCAL_MACHINE, windows::Win32::System::Registry::, subkey)
            win32_get_system_programdata_folder_path()?
                .join(vendor_name())
                .join(fmt_program_name_ascii())
        }
        #[cfg(all(not(target_os = "windows"), not(target_os = "linux")))]
        {
            return Err("Unsupported Operating system".into());
            std::path::Path::new("s").to_path_buf()
        }
    };

    let program_name = fmt_program_name_ascii();
    let vendor = vendor_name();
    Ok(config_base_path.join(vendor).join(program_name))
}
/// creates the service directory. requires ROOT permissions on linux
pub fn create_service_config_directory() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let config_dir = get_service_config_directory().map_err(|e| {
        format!("Failed to get service config directory while attempting to create it: {e}")
    })?;
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir)
}

/// checks if the config file exists. use before creating
pub fn config_file_exists() -> Result<bool, Box<dyn std::error::Error>> {
    let config_dir = get_service_config_directory()?;
    let config_file_path = config_dir.join(CONFIG_FILENAME);
    let exists = std::fs::exists(config_dir)?;
    Ok(exists)
}

/// creates the config file. equivelent to touch 'filename
/// overwrites the file if it exists
pub fn open_config_file() -> Result<std::fs::File, Box<dyn std::error::Error>> {
    // ensure the directory exists
    let config_dir = create_service_config_directory()?;
    let config_file_path = config_dir.join(CONFIG_FILENAME);
    let f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(config_file_path)?;
    Ok(f)
}

/// opens the config file either at the specified path or the default config path
pub fn figment(
    path: Option<&std::path::Path>,
) -> Result<figment::Figment, Box<dyn std::error::Error>> {
    let path = {
        if let Some(path) = path {
            path.join(CONFIG_FILENAME).to_path_buf()
        } else {
            let config_dir = get_service_config_directory()?;
            let config_file_path = config_dir.join(CONFIG_FILENAME);
            config_file_path
        }
    };
    let figment = figment::Figment::new().merge(figment::providers::Toml::file(path));
    Ok(figment)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    pub vault_data_directory: PathBuf,
    pub vault_config_direcory: PathBuf,
}
impl Config {
    /// creates a default config as follows:
    /// on linux: the default data directory (for encrypted files) will be /var/lib/{vendor}/{app}
    /// on windows: the default data directory (for encrypted files) will be %PROGRAMDATA%/{vendor}/app
    ///
    /// on linux: the default config directory (for vault configuration) will be /etc/{vendor}/{app}
    /// on windows: the default config directory (for vault configuration) will be %PROGRAMDATA\\{vendor}\\{app}
    ///
    /// the program should always try to read the default config directory for config.toml first, then read the configured data dir
    pub fn try_default() -> Result<Self, Box<dyn std::error::Error>> {
        // the default data directory for the app
        let data_dir = get_service_data_directory()?;
        let config_dir = get_service_data_directory()?;
        let s = Self {
            vault_data_directory: data_dir,
            vault_config_direcory: config_dir,
        };
        Ok(s)
    }
    pub fn try_load<P: AsRef<std::path::Path>>(p: P) -> Result<Self, Box<dyn std::error::Error>> {
        let figment = self::figment(Some(p.as_ref()))?;
        figment.extract().map_err(|e| e.into())
    }
    /// loads the file at the default data directory, then merges it with the file inside the config
    pub fn try_load_default_merged() -> Result<Self, Box<dyn std::error::Error>> {
        // load from the default directory first, then load from the configured directory
        let figment = self::figment(None)?;
        let config: Config = figment.extract()?;
        // load the subconfig file
        let merged: Config = figment
            .merge(figment::providers::Toml::file(
                config.vault_config_direcory.join(CONFIG_FILENAME),
            ))
            .extract()
            .unwrap_or(config);
        Ok(merged)
    }

    /// overwrites the config file to the destination specified
    pub fn write<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = toml::to_string(self)?;
        std::fs::File::create(path.as_ref())?.write_all(buffer.as_bytes())?;
        Ok(())
    }
}
