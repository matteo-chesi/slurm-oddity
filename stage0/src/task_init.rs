use std::error::Error;
use std::ffi::OsStr;
use std::process::Command;
use users::get_current_uid;
use libc::{self, gid_t, uid_t};

use slurm_spank::SpankHandle;

use crate::{SpankStage0, get_log_dirpath};
use crate::config::{Stage0Config, expand_vars_string};

pub(crate) fn task_init_adjust(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {
    task_init_sethomevar(plugin, spank)?;
    task_init_adjust_logdir(plugin, spank)?;
    task_init_adjust_config(plugin, spank)?;
    Ok(())
}

fn task_init_sethomevar(
    _plugin: &mut SpankStage0,
    _spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {
    let uid = get_current_uid();

    let getent_out = match Command::new("getent")
                       .args(&["passwd", uid.to_string().as_str()])
                       .output() {
        Ok(process) => process,
        Err(err)    => return Err(format!("Running command error: getent passwd {uid}: {err}").into()),
    };

    if ! getent_out.status.success() {
        let code = getent_out.status.code().unwrap();
        return Err(format!("Exit command error: getent passwd {uid} returned: {code}").into());
    };

    let getent_stdout = match std::string::String::from_utf8(getent_out.stdout) {
        Ok(out)  => out,
        Err(err) => return Err(format!("Translating output command error: getent passwd {uid}: {err}").into()),
    };

    let getent_stdout_vec: Vec<&str> = getent_stdout.split(':').collect();
    let value = match getent_stdout_vec.get(5) {
        Some(s) => std::path::PathBuf::from(s),
        None => return Err(format!("Output command error: getent passwd {uid}, cannot find homedir field.").into()),
    };

    let key = "HOME";
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }
    Ok(())

}

fn task_init_adjust_logdir(
    _plugin: &mut SpankStage0,
    _spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    let key = "SLURM_STAGE0_LOGDIR";
    let value = get_log_dirpath();
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }
    Ok(())
}

fn task_init_adjust_config(
    plugin: &mut SpankStage0,
    _spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    let new_config = Stage0Config {
        stage1_user_path: expand_vars_string(plugin.config.stage1_user_path.clone())?,
        stage1_system_path: expand_vars_string(plugin.config.stage1_system_path.clone())?,
    };

    let mut key = "SLURM_STAGE1_USER_BIN";
    let mut value = new_config.stage1_user_path.clone();
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }

    key = "SLURM_STAGE1_SYSTEM_BIN";
    value = new_config.stage1_system_path.clone();
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }
    plugin.config = new_config;

    Ok(())
}

pub(crate) fn task_init_set_job_uid(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    let key = "SLURM_STAGE0_UID";
    let job_uid = spank.job_uid()?;
    unsafe {
        std::env::set_var(key, OsStr::new(&job_uid.to_string()));
    }

    let cur_uid = get_current_uid();
    if cur_uid == 0 && cur_uid != job_uid {
        unsafe {
            let rc = libc::setuid(job_uid);
            if rc != 0 {
                return Err(format!("Error switching user id from {cur_uid} to {job_uid}").into());
            }
            plugin.state.old_uid = Some(cur_uid);
        }
    }
    Ok(())
}
pub(crate) fn task_init_revert_uid(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {
    let cur_uid = get_current_uid();
    
    let old_uid = match plugin.state.old_uid {
        Some(uid) => uid,
        None => return Ok(()),
    };

    if cur_uid != 0 && old_uid == 0 {
        unsafe {
            let rc = libc::setuid(old_uid);
            if rc != 0 {
                return Err(format!("Error reverting user id from {cur_uid} to {old_uid}").into());
            }
            plugin.state.old_uid = None;
        }
    }
    Ok(())
}

