use std::collections::HashMap;
use std::env::set_current_dir;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;
//use std::process::Output;
//use serde_json::{Value};
use tracing::{info};
use slurm_spank::{SpankError, SpankHandle};

use crate::{SpankStage0, state::get_local_task_id};

use cfg_if;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        pub type PtrT = u8;
    } else {
        pub type PtrT = i8;
    }
}


pub(crate) fn container_join_from_iodata(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    let iodata = plugin.iodata.clone().unwrap();
    let env;
    let pid;
    let workdir;

    if iodata.exchange.container.is_some() {
        let container = iodata.exchange.container.unwrap();
        pid = container.pid;
        env = container.env;
        workdir = container.workdir;
    } else {
        return Err("stage1 returned no container data".into());
    }

    info!("CONTAINER_PID: {pid}");
    info!("EDF_ENV: {:#?}", env);
    info!("EDF_WORKDIR: {}", workdir);

    info!("container_join_from_iodata 1");
    let ret = container_join(plugin, spank, pid, workdir, env);
    info!("container_join_from_iodata 2");
    ret
}

/*
pub(crate) fn container_join_from_stage1_output(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
    output: String,
) -> Result<(), Box<dyn Error>> {
    info!("container_join_from_stage1_output 1");
    info!("OUTPUT: {:#?}",output);
    let container_pid = get_container_pid_from_json_output(&output);
    let pid;
    if container_pid.is_ok() {
        pid = container_pid.unwrap();
        info!("CONTAINER_PID: {pid}");
    } else {
        return Err("stage1 returned bad output.".into());
    }

    info!("container_join_from_stage1_output 2");
    let edf_env = get_edf_env_from_json_output(&output);
    let env;
    if edf_env.is_ok() {
        env = edf_env.unwrap();
        info!("EDF_ENV: {:#?}", env);
    } else {
        return Err("missing edf env from stage1 output.".into());
    }

    info!("container_join_from_stage1_output 3");
    let edf_workdir = get_edf_workdir_from_json_output(&output);
    let workdir;
    if edf_workdir.is_ok() {
        workdir = edf_workdir.unwrap();
        info!("EDF_WORKDIR: {}", workdir);
    } else {
        return Err("missing edf workdir from stage1 output.".into());
    }

    info!("container_join_from_stage1_output 4");
    let ret = container_join(plugin, spank, pid, workdir, env);
    info!("container_join_from_stage1_output 5");
    ret
}
*/

pub(crate) fn container_join(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
    pid: u32,
    edf_workdir: String,
    edf_env: HashMap<String, String>)
-> Result<(), Box<dyn Error>> {
    info!("container_join 1");
    inner_unsafe_container_join(pid)?;
    info!("container_join 2");
    inner_container_wait_cwd(plugin, spank, pid)?;
    info!("container_join 3");
    inner_container_import_env(plugin, spank, pid, edf_env)?;
    info!("container_join 4");
    inner_container_set_workdir(plugin, spank, pid, edf_workdir)?;
    info!("container_join 5");
    return Ok(());
}
/*
pub(crate) fn get_container_pid_from_json_output(output: &str) -> Result<u32, Box<dyn Error>> {
    let mut value: Value = match serde_json::from_str(output) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("couldn't parse command output as a valid json: {e}").into());
        },
    };

    let root = match value.as_object_mut() {
        Some(obj) => obj,
        None => {
            return Err(format!("couldn't find root object in json command output").into());
        },
    };

    let run_val = match root.get_mut("run") {
        Some(v) => v,
        None => {
            return Err(format!("couldn't find run value in json command output").into());
        },
    };

    let run_obj = match run_val.as_object_mut() {
        Some(obj) => obj,
        None => {
            return Err(format!("run is not an object in json command output").into());
        },
    };

    let pid_val = match run_obj.get("pid") {
        Some(v) => v,
        None => {
            return Err(format!("couldn't find pid value in json command output").into());
        },
    };

    let pid = match pid_val.as_u64() {
        Some(num) => {
            if num <= u32::MAX as u64 {
                num as u32
            } else {
                return Err(format!("pid {num} cannot be converted as u32").into());
            }
        },
        None => {
            return Err(format!("pid cannot be converted as u64 in json command output").into());
        },
    };

    return Ok(pid);
}

pub(crate) fn get_edf_env_from_json_output(output: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut value: Value = match serde_json::from_str(&output) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("couldn't parse command output as a valid json: {e}").into());
        },
    };

    let root = match value.as_object_mut() {
        Some(obj) => obj,
        None => {
            return Err(format!("couldn't find root object in json command output").into());
        },
    };

    let edf_val = match root.get_mut("edf") {
        Some(v) => v,
        None => {
            return Err(format!("couldn't find edf value in json command output").into());
        },
    };

    let edf_obj = match edf_val.as_object_mut() {
        Some(obj) => obj,
        None => {
            return Err(format!("edf is not an object in json command output").into());
        },
    };

    let edf_env_val = match edf_obj.get("env") {
        Some(v) => v,
        None => {
            return Err(format!("edf.env is cannot be parsed from json command output").into());
        },
    };

    let edf_env_obj = match edf_env_val.as_object() {
        Some(obj) => obj,
        None => {
            return Err(format!("edf.env is not an object in json command output").into());
        },
    };

    let mut edf_env: HashMap<String, String> = HashMap::from([]);

    for (k, v) in edf_env_obj.iter() {
        edf_env.insert(k.to_string(), v.to_string());
    }

    return Ok(edf_env);
}

pub(crate) fn get_edf_workdir_from_json_output(output: &str) -> Result<String, Box<dyn Error>> {
    let mut value: Value = match serde_json::from_str(&output) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("couldn't parse command output as a valid json: {e}").into());
        },
    };

    let root = match value.as_object_mut() {
        Some(obj) => obj,
        None => {
            return Err(format!("couldn't find root object in json command output").into());
        },
    };

    let edf_val = match root.get_mut("edf") {
        Some(v) => v,
        None => {
            return Err(format!("couldn't find edf value in json command output").into());
        },
    };

    let edf_obj = match edf_val.as_object_mut() {
        Some(obj) => obj,
        None => {
            return Err(format!("edf is not an object in json command output").into());
        },
    };

    let edf_workdir_val = match edf_obj.get("workdir") {
        Some(v) => v,
        None => {
            return Err(format!("edf.workdir is cannot be parsed from json command output").into());
        },
    };

    let edf_workdir = match edf_workdir_val.as_str() {
        Some(obj) => obj,
        None => {
            return Err(format!("edf.workdir is not a string in json command output").into());
        },
    };

    return Ok(String::from(edf_workdir));
}
*/
fn inner_unsafe_container_join(pid: u32) -> Result<(), Box<dyn Error>> {
    unsafe {
        // First collect file descriptors for relevant namespaces

        // User namespace
        let userns_path = format!("/proc/{pid}/ns/user");
        let userns_path_c = userns_path.clone() + "\0";
        let userns_path_ptr: *const PtrT = userns_path_c.as_ptr() as *const PtrT;

        let userns_fd = libc::open(userns_path_ptr, libc::O_RDONLY | libc::O_CLOEXEC);
        if userns_fd < 0 {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap();
            let msg = format!("failed to open userns file \"{userns_path}\", error: {errno}");
            return Err(msg.into());
        }

        // Mount namespace
        let mntns_path = format!("/proc/{pid}/ns/mnt");
        let mntns_path_c = mntns_path.clone() + "\0";
        let mntns_path_ptr: *const PtrT = mntns_path_c.as_ptr() as *const PtrT;

        let mntns_fd = libc::open(mntns_path_ptr, libc::O_RDONLY | libc::O_CLOEXEC);
        if mntns_fd < 0 {
            let msg = format!("failed to open mount namespace file: {mntns_path}");
            return Err(msg.into());
        }

        // Then join all relevant namespaces

        // Join user namespace
        let ret = libc::setns(userns_fd, libc::CLONE_NEWUSER);
        if ret < 0 {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap();
            let msg = format!("failed to join user namespace, error: {errno}");
            return Err(msg.into());
        }

        // Join mount namespace
        let ret = libc::setns(mntns_fd, libc::CLONE_NEWNS);
        if ret < 0 {
            return Err("failed to join mount namespace".into());
        }
    }

    Ok(())
}

fn inner_container_wait_cwd(plugin: &mut SpankStage0, _spank: &mut SpankHandle, pid: u32) -> Result<(), Box<dyn Error>> {
    let cwd = format!("/proc/{pid}/cwd");

    let mut attempts: u32 = 0;
    let pause = std::time::Duration::from_millis(100);
    let max_attempts: u32 = 600;

    loop {
        // Validate the cwd symlink resolves to an actual cwd. If not, return failure string.
        let failure: Option<String> = match std::fs::read_link(&cwd) {
            Ok(target) => {
                if target.is_dir() {
                    None
                } else {
                    Some(format!("cwd link resolved to non-dir target {target:?}"))
                }
            }
            Err(e) => Some(format!("couldn't read cwd link {cwd}: {e}")),
        };

        // Success case
        if failure.is_none() {
            break;
        }

        // Go for retry
        attempts += 1;
        let failure = failure.unwrap();

        // Log first and every 50 retries to limit log spam
        if attempts == 1 || attempts % 50 == 0 {
            info!("task {} - {failure}, waiting and retrying", get_local_task_id(&plugin.state));
        }

        // Fail with error after max attempts
        if attempts >= max_attempts {
            let msg = format!(
                "failed to resolve container cwd via {cwd} after {attempts} attempts: {failure}. \
This can happen if the container is not started with host PID namespace (podman --pidns=host), \
so the host cannot access /proc/<pid>/cwd for the container process."
            );
            info!("task {} - {msg}", get_local_task_id(&plugin.state));
            return Err(msg.into());
        }

        std::thread::sleep(pause);
    }

    Ok(())

}

fn inner_container_import_env(
    _plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
    pid: u32,
    env: HashMap<String, String>)
-> Result<(), Box<dyn Error>> {
    let container_deny_env = vec!["LANG", "LANGUAGE", "LC_ALL"];
    let edf_env = env;

    for dvar in container_deny_env {
        if !edf_env.contains_key(dvar) {
            match spank.unsetenv(dvar) {
                Ok(_) => {}
                Err(e) => {
                    info!("failed to unset {dvar}: {e}");
                    return Err(Box::new(e));
                }
            }
        }
    }

    let environ_path = format!("/proc/{pid}/environ");
    let environ = Path::new(&environ_path);

    let environ_file = match File::open(&environ) {
        Ok(f) => f,
        Err(e) => {
            info!("couldn't open environ {environ_path}: {e}");
            return Err(Box::new(e));
        }
    };

    let mut container_vars = HashMap::new();
    let lines = BufReader::new(environ_file).split(0);
    // Consumes the iterator, returns an (Optional) String
    for line in lines.map_while(Result::ok) {
        let string = String::from_utf8(line)?;
        match string.split_once('=') {
            Some((key, value)) => {
                //spank_log_user!("{} = {}", key, value);
                container_vars.insert(String::from(key), String::from(value));
            }
            None => {
                let msg = format!("couldn't parse environ value {string}");
                info!("{msg}");
                return Err(msg.into());
            }
        }
    }

    let mut unset_keys = vec![];
    for (key, value) in edf_env.iter() {
        if value == "" {
            unset_keys.push(key);
            continue;
        }
        let overwrite = true;

        match spank.setenv(key, value, overwrite) {
            Ok(ok) => ok,
            Err(SpankError::EnvExists(_)) => (),
            Err(e) => {
                let msg = format!("couldn't set env {key}={value}: {e}");
                info!("{msg}");
                return Err(Box::new(e));
            }
        }
    }

    for (key, value) in container_vars.iter() {
        if unset_keys.contains(&key) {
            continue;
        }

        let mut overwrite = true;
        if edf_env.contains_key(key) {
            overwrite = false;
        }

        match spank.setenv(key, value, overwrite) {
            Ok(ok) => ok,
            Err(SpankError::EnvExists(_)) => (),
            Err(e) => {
                let msg = format!("couldn't set env {key}={value}: {e}");
                info!("{msg}");
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}

fn inner_container_set_workdir(
    plugin: &mut SpankStage0,
    _spank: &mut SpankHandle,
    pid: u32,
    workdir: String,
) -> Result<(), Box<dyn Error>> {
    let mut new_workdir = workdir;

    if new_workdir == "" {
        new_workdir = format!("/proc/{pid}/cwd");
    }

    let msg = format!(
        "task {} - changing workdir to {new_workdir}",
        get_local_task_id(&plugin.state)
    );
    info!("{msg}");

    let new_cwd = Path::new(&new_workdir);
    Ok(set_current_dir(&new_cwd)?)
}
