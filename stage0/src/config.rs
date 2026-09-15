use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use slurm_spank::{Context, SpankHandle};
use tracing::{info, error};

use crate::{SpankStage0, spank_getenv};

const CONFIGFILE_PATH: &str = "/etc/slurm-oddity-stage0.conf";

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RawStage0Config {
    stage1_user_path: Option<String>,
    stage1_system_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Stage0Config {
    #[serde(default = "get_default_stage1_user_path")]
    pub stage1_user_path: String,
    #[serde(default = "get_default_stage1_system_path")]
    pub stage1_system_path: String,
}

fn get_default_stage1_user_path() -> String {
    return String::from("$HOME/.local/share/slurm-oddity/bin/stage1");
}

fn get_default_stage1_system_path() -> String {
    return String::from("/usr/bin/stage1");
}

impl From<RawStage0Config> for Stage0Config {
    fn from(r: RawStage0Config) -> Self {
        Stage0Config {
            stage1_user_path: match r.stage1_user_path {
                Some(s) => s,
                None => get_default_stage1_user_path(),
            },
            stage1_system_path: match r.stage1_system_path {
                Some(s) => s,
                None => get_default_stage1_system_path(),
            },
        }
    }
}

pub fn load_config() -> Result<Stage0Config, Box<dyn Error>> {
    load_configfile_path(None)
}

pub fn load_configfile_path(
    configfile_path_option: Option<PathBuf>,
) -> Result<Stage0Config, Box<dyn Error>> {
    let configfile_path = match configfile_path_option {
        Some(path) => path,
        None => PathBuf::from(CONFIGFILE_PATH),
    };

    let r = match configfile_path.exists() {
        true => load_raw_config_from_file(&configfile_path)?,
        false => {
            let mut x = RawStage0Config{
                stage1_user_path: Some(get_default_stage1_user_path()),
                stage1_system_path: Some(get_default_stage1_system_path()),
            };
            expand_raw_config_fields(&mut x)?;
            x
        },
    };

    //let r = load_raw_config_from_file(&configfile_path)?;
    let c = Stage0Config::from(r);
    Ok(c)
}

fn load_raw_config_from_file(
    configfile_path: &Path,
) -> Result<RawStage0Config, Box<dyn Error>> {
    let toml_content = std::fs::read_to_string(configfile_path)?;
    let toml_value = toml::from_str(&toml_content)?;
    let mut r: RawStage0Config = toml_value;
    expand_raw_config_fields(&mut r)?;
    Ok(r)
}

fn expand_raw_config_fields(
    r: &mut RawStage0Config,
) -> Result<(), Box<dyn Error>> {
    expand_raw_option_string(&mut r.stage1_user_path)?;
    expand_raw_option_string(&mut r.stage1_system_path)?;
    Ok(())
}

fn expand_raw_option_string(
    optstr: &mut Option<String>,
) -> Result<(), Box<dyn Error>> {
    if optstr.is_some() {
        let original = optstr.clone().unwrap();
        let updated = expand_vars_string(original.clone())?;
        *optstr = Some(updated);
    }
    Ok(())
}

pub(crate) fn expand_vars_string(s: String) -> Result<String, Box<dyn Error>> {
    match shellexpand::env(&s) {
        Ok(ok) => return Ok(ok.to_string()),
        Err(_) => return Ok(s), 
    };
}

pub(crate) fn dispatch_load_config(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    match spank.context()? {
        Context::Local | Context::Allocator | Context::Slurmd => {
            match local_load_config(plugin, spank) {
                Ok(_) => (),
                Err(_) => {
                    return Ok(());
                }
            }
        },
        Context::Remote => {
            match remote_load_config(plugin, spank) {
                Ok(_) => (),
                Err(_) => {
                    return Ok(());
                }
            }
        }
        _ => {
                return Ok(());
        },
    }

    Ok(())
}

pub(crate) fn local_load_config(
    plugin: &mut SpankStage0,
    _spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Error on configuration loading: {e}");
            return Err(e);
        }
    };

    let mut key = "SLURM_STAGE1_USER_BIN";
    let mut value = config.stage1_user_path.clone();
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }

    key = "SLURM_STAGE1_SYSTEM_BIN";
    value = config.stage1_system_path.clone();
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }
    plugin.config = config.clone();

    info!("STAGE1_USER_PATH: {}", plugin.config.stage1_user_path);
    info!("STAGE1_SYSTEM_PATH: {}", plugin.config.stage1_system_path);

    Ok(())
}

pub(crate) fn remote_load_config(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    let mut key = "SLURM_STAGE1_USER_BIN";
    let mut value = spank_getenv(spank, key);
    let stage1_user_path = value.to_string();

    key = "SLURM_STAGE1_SYSTEM_BIN";
    value = spank_getenv(spank, key);
    let stage1_system_path = value.to_string();

    let config = Stage0Config {
        stage1_user_path: stage1_user_path,
        stage1_system_path: stage1_system_path,
    };
    plugin.config = config;

    info!("STAGE1_USER_PATH: {}", plugin.config.stage1_user_path);
    info!("STAGE1_SYSTEM_PATH: {}", plugin.config.stage1_system_path);

    Ok(())
}
