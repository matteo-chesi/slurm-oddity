use std::error::Error;
use std::fs::create_dir_all;
use std::os::unix::fs::chown;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use const_format::formatcp;
use serde::{Serialize, Deserialize};

use slurm_spank::{Plugin, SLURM_VERSION_NUMBER, SPANK_PLUGIN, SpankHandle};
use crate::args::Stage0Args;
use crate::config::Stage0Config;
use crate::state::Stage0State;

pub(crate) use crate::log::{
    ErrorDestination,
    error_destination,
    remote_log,
    init_log_file,
    format_error_chain,
    get_log_dirpath,
    set_panic_hook
};
pub(crate) use crate::stage1::{run_stage1, remote_run_stage1_new};
pub(crate) use crate::state::{set_local2remote_env_var, get_job_env};

pub mod args;
pub mod config;
pub mod containers;
pub mod dispatch;
pub mod log;
pub mod stage1;
pub mod state;

//pub(crate) const SLURM_BATCH_SCRIPT: u32 = 0xfffffffb;
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const PLUGIN_NAME: &str = "stage0";
pub(crate) const APP_NAME: &str = formatcp!("{}-{}", PLUGIN_NAME, VERSION);
pub(crate) const LOG_PATH: &str = formatcp!("${{HOME}}/.local/share/{}/log", PLUGIN_NAME);
pub(crate) const DATETIME_FORMAT: &str = "%Y%m%d";
pub(crate) const LOCAL_LOG_FILENAME: &str = "${DATETIME}_${CLUSTER_NAME}/local_${HOSTNAME}.log";
pub(crate) const REMOTE_LOG_FILENAME: &str = "${DATETIME}_${CLUSTER_NAME}/job_${SLURM_JOB_ID}/${HOSTNAME}.log";
pub(crate) const CACHE_PATH: &str = "${HOME}/.local/share/cosmodrome/cache";
pub(crate) const LOCAL2REMOTE_VARNAME: &str = "SLURM_STAGE0_LOCAL2REMOTE_DATA";
pub(crate) const LOCAL2REMOTE_FILENAME: &str = "local2remote_data.json";

SPANK_PLUGIN!(b"stage0", SLURM_VERSION_NUMBER, SpankStage0);

#[derive(Serialize, Default)]
struct SpankStage0 {
    log_file: PathBuf,
    args: Stage0Args,
    config: Stage0Config,
    state: Stage0State,
}

#[derive(Deserialize, Serialize, Default)]
struct ConsoleOutput {
    console_out: String,
}

pub(crate) fn get_plugin_name() -> String {
    return String::from(PLUGIN_NAME);
}

pub(crate) fn get_versioned_plugin_name() -> String {
    return String::from(format!("{}-v{}", PLUGIN_NAME, VERSION));
}

pub(crate) fn spank_getenv(spank: &mut SpankHandle, var: &str) -> String {
    match spank.getenv(var) {
        Ok(r) => match r {
            Some(v) => v,
            None => String::from(""),
        },
        Err(_) => String::from(""),
    }
}

pub(crate) fn create_dir_path(plugin: &mut SpankStage0, dir_path: &Path) -> Result<(), Box<dyn Error>> {
    let path = dir_path;

    if ! path.exists() {

        match path.parent() {
            Some(pp) => {
                if ! pp.exists() {
                    create_dir_path(plugin, pp)?;
                }
            },
            None => {},
        };
        create_dir_all(path)?;

        // Set ownership
        if (plugin.state.uid.is_some() && 
            ( std::fs::metadata(&path).unwrap().uid() != plugin.state.uid.unwrap())) ||
            ( plugin.state.gid.is_some() && 
            ( std::fs::metadata(&path).unwrap().gid() != plugin.state.gid.unwrap())) {
                match chown(&path, plugin.state.uid, plugin.state.gid) {
                    Ok(_) => (),
                    Err(_) => {
                        return Ok(());
                    },
                }
        }
    }
    Ok(())
}

/*
pub(crate) fn spank_setenv(spank: &mut SpankHandle, name: &str, value: &str) -> Result<(),String> {
    match spank.setenv(name, value, true) {
        Ok(r) => Ok(r),
        Err(_) => { return Err(format!("Cannot set variable {}={}", name, value)); },
    }
}
*/
