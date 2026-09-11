//use std::error::Error;
//use std::fs::create_dir_all;
//use std::os::unix::fs::chown;
//use std::os::unix::fs::MetadataExt;
use std::path::{/*Path,*/ PathBuf};
use const_format::formatcp;
use serde::{Serialize, Deserialize};

use slurm_spank::{Plugin, SLURM_VERSION_NUMBER, SPANK_PLUGIN, SpankHandle};
use crate::args::{Stage0Args, get_args};
use crate::config::{Stage0Config, load_config};
use crate::iodata::{
    IOData,
    get_iodata,
    get_iodata_from_str,
    get_job_arg,
    get_job_env,
    update_iodata,
    set_local2remote_env_var_from_iodata
};
use crate::state::Stage0State;

pub(crate) use crate::log::{
    ErrorDestination,
    error_destination,
    init_log_file,
    format_error_chain,
    get_log_dirpath,
    get_command_log_filepath,
    set_panic_hook,
};
pub(crate) use crate::stage1::{run_stage1_new2};
//pub(crate) use crate::state::{set_local2remote_env_var};

pub mod args;
pub mod config;
pub mod containers;
pub mod dispatch;
pub mod log;
pub mod iodata;
pub mod stage1;
pub mod state;

//pub(crate) const SLURM_BATCH_SCRIPT: u32 = 0xfffffffb;
pub(crate) const NAME: &str = "slurm-oddity";
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const PLUGIN_NAME: &str = "stage0";
pub(crate) const COMMAND_NAME: &str = "stage1";
pub(crate) const APP_NAME: &str = formatcp!("{}-{}", PLUGIN_NAME, VERSION);
pub(crate) const LOG_PATH: &str = formatcp!("${{HOME}}/.local/share/{}/log", NAME);
pub(crate) const DATETIME_FORMAT: &str = "%Y%m%d";
pub(crate) const LOCAL_LOG_FILENAME: &str = formatcp!("${{DATETIME}}_${{CLUSTER_NAME}}/local_${{HOSTNAME}}_{}.log", PLUGIN_NAME);
pub(crate) const REMOTE_LOG_FILENAME: &str = formatcp!("${{DATETIME}}_${{CLUSTER_NAME}}/job_${{SLURM_JOB_ID}}/${{SLURM_TOPOLOGY_ADDR}}_{}.log", PLUGIN_NAME);
pub(crate) const COMMAND_LOCAL_LOG_FILENAME: &str = formatcp!("${{DATETIME}}_${{CLUSTER_NAME}}/local_${{HOSTNAME}}_{}.log", COMMAND_NAME);
pub(crate) const COMMAND_REMOTE_LOG_FILENAME: &str = formatcp!("${{DATETIME}}_${{CLUSTER_NAME}}/job_${{SLURM_JOB_ID}}/${{SLURM_TOPOLOGY_ADDR}}_{}.log", COMMAND_NAME);
//pub(crate) const CACHE_PATH: &str = formatcp!("${{HOME}}/.local/share/{}/cache", NAME);
pub(crate) const LOCAL2REMOTE_VARNAME: &str = "SLURM_STAGE0_LOCAL2REMOTE_DATA";
//pub(crate) const LOCAL2REMOTE_FILENAME: &str = "local2remote_data.json";

SPANK_PLUGIN!(b"stage0", SLURM_VERSION_NUMBER, SpankStage0);

#[derive(Serialize, Default)]
struct SpankStage0 {
    log_file: PathBuf,
    args: Stage0Args,
    config: Stage0Config,
    state: Stage0State,
    pub(crate) iodata: Option<IOData>,
}

#[derive(Deserialize, Serialize, Default)]
struct ConsoleOutput {
    console_out: String,
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

/*
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
*/
