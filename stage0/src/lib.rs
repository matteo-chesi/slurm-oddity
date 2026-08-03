use serde::Serialize;

use slurm_spank::{Plugin, SLURM_VERSION_NUMBER, SPANK_PLUGIN, SpankHandle};
use crate::args::Stage0Args;
use crate::config::Stage0Config;
use crate::state::Stage0State;

pub(crate) use crate::log::{log, remote_log, get_log_dirpath};
pub(crate) use crate::stage1::run_stage1;
pub(crate) use crate::task_init::task_init_adjust;

pub mod args;
pub mod config;
pub mod dispatch;
pub mod log;
pub mod stage1;
pub mod state;
pub mod task_init;

//pub(crate) const SLURM_BATCH_SCRIPT: u32 = 0xfffffffb;
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const PLUGIN_NAME: &str = "stage0";

SPANK_PLUGIN!(b"stage0", SLURM_VERSION_NUMBER, SpankStage0);

#[derive(Serialize, Default)]
struct SpankStage0 {
    args: Stage0Args,
    config: Stage0Config,
    state: Stage0State,
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => ({
        slurm_spank::spank_log(slurm_spank::LogLevel::Error, &format!("[{}] {}", $crate::get_plugin_name(), &format!($($arg)*)));
    })
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
