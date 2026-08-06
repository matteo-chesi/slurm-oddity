use std::error::Error;
use std::ffi::OsStr;
use serde::{Deserialize, Serialize};

use slurm_spank::{Context, SpankHandle};

use crate::{SpankStage0, get_log_dirpath, log, remote_log, spank_getenv};

#[derive(Serialize, Deserialize)]
pub(crate) struct Stage0State {
    pub(crate) enabled: bool,
    pub(crate) old_uid: Option<u32>,
    pub(crate) old_gid: Option<u32>,
    pub(crate) stage0_logdir_path: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) uid: Option<u32>,
    pub(crate) gid: Option<u32>,
    pub(crate) jobid: Option<String>,
}

impl Default for Stage0State {
    fn default() -> Self {
        Stage0State {
            enabled: true,
            old_uid: None,
            old_gid: None,
            stage0_logdir_path: None,
            username: None,
            uid: None,
            gid: None,
            jobid: None,
        }
    }
}

pub(crate) fn dispatch_load_state(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    match spank.context()? {
        Context::Local | Context::Allocator => {
            match local_load_state(plugin, spank) {
                Ok(_) => (),
                Err(_) => {
                    return Ok(());
                }
            }
        },
        Context::Remote => {
            match remote_load_state(plugin, spank) {
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

pub(crate) fn local_load_state(
    plugin: &mut SpankStage0,
    _spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    let mut key = "SLURM_STAGE0_LOGDIR";
    let mut value = get_log_dirpath();
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }
    plugin.state.stage0_logdir_path = Some(value);

    let logdir = match &plugin.state.stage0_logdir_path {
        Some(l) => l,
        None => &String::from("None"),
    };
    log(&format!("STAGE0_LOGDIR: {}", logdir));

    key = "SLURM_STAGE0_USERNAME";
    value = whoami::username().unwrap();
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }
    plugin.state.username = Some(value);
    
    let username = match &plugin.state.username {
        Some(u) => u,
        None => &String::from("None"),
    };
    log(&format!("STAGE0_USERNAME: {}", username));

    Ok(())
}

pub(crate) fn remote_load_state(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    let mut key = "SLURM_STAGE0_JOBID";
    plugin.state.jobid = match spank.job_id() {
        Ok(id) => {
            let strid = id.to_string();
            unsafe {std::env::set_var(key, OsStr::new(&strid));}
            remote_log(plugin, spank, &format!("STAGE0_JOBID: {}", strid));
            Some(strid)
        },
        Err(_) => {
            unsafe {std::env::remove_var(key);}
            None
        },
    };
    
    key = "SLURM_STAGE0_LOGDIR";
    let mut value = spank_getenv(spank, key).to_string();
    plugin.state.stage0_logdir_path = match value.as_str() {
        "None" => {
            unsafe {std::env::remove_var(key);}
            None
        },
        _ => {
            let mut log_dirpath = std::path::PathBuf::from(&value);
            if plugin.state.jobid.is_some() {
                let dirname = format!("job_{}", &plugin.state.jobid.clone().unwrap());
                log_dirpath = log_dirpath.join(std::path::PathBuf::from(&dirname));
            }
            unsafe {std::env::set_var(key, OsStr::new(&log_dirpath));}
            Some(log_dirpath.clone().into_os_string().into_string().unwrap())
        },
    };
    remote_log(plugin, spank, &format!("STAGE0_LOGDIR: {}", value));

    key = "SLURM_STAGE0_USERNAME";
    value = spank_getenv(spank, key).to_string();
    plugin.state.username = match value.as_str() {
        "None" => {
            unsafe {std::env::remove_var(key);}
            None
        },
        _ => {
            unsafe {std::env::set_var(key, OsStr::new(&value));}
            Some(value.clone())
        },
    };
    remote_log(plugin, spank, &format!("STAGE0_USERNAME: {}", value));

    key = "SLURM_STAGE0_UID";
    plugin.state.uid = match spank.job_uid() {
        Ok(uid) => {
            let struid = uid.to_string();
            unsafe {std::env::set_var(key, OsStr::new(&struid));}
            remote_log(plugin, spank, &format!("STAGE0_UID: {}", struid));
            Some(uid)
        },
        Err(_) => {
            unsafe {std::env::remove_var(key);}
            None
        },
    };

    key = "SLURM_STAGE0_GID";
    plugin.state.gid = match spank.job_gid() {
        Ok(gid) => {
            let strgid = gid.to_string();
            unsafe {std::env::set_var(key, OsStr::new(&strgid));}
            remote_log(plugin, spank, &format!("STAGE0_GID: {}", strgid));
            Some(gid)
        },
        Err(_) => {
            unsafe {std::env::remove_var(key);}
            None
        },
    };

    Ok(())
}
