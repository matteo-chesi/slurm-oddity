use std::env::home_dir;
use std::fs::OpenOptions;
use std::fs::create_dir_all;
use std::io::Write;
use std::os::unix::fs::{chown, MetadataExt};
use std::path::PathBuf;
use chrono::Utc;
use chrono_tz::Tz;
use gethostname::gethostname;
use nix::unistd::{Uid, setfsuid};

use slurm_spank::{SpankHandle};

use crate::{get_versioned_plugin_name, spank_getenv};
use crate::SpankStage0;

pub(crate) fn log(arg: &str) {

        let log_dirpath = std::path::PathBuf::from(get_log_dirpath());
        let hostname = gethostname()
            .into_string()
            .unwrap_or(String::from("unknown_hostname"));
        let log_node_dirpath = log_dirpath.join(std::path::PathBuf::from(hostname.clone()));
        stage0_log(log_node_dirpath, hostname, arg);
}

pub(crate) fn get_log_dirpath() -> String {
    let home_dir = home_dir().unwrap();
    let log_dirpath = home_dir.join(std::path::PathBuf::from("cosmodrome"));
    return log_dirpath.into_os_string().to_string_lossy().to_string();
}

pub(crate) fn remote_log(plugin: &mut SpankStage0, spank: &mut SpankHandle, arg: &str) {

    let key = "SLURM_STAGE0_LOGDIR";
    let mut value = spank_getenv(spank, key);
    if value.is_empty() {
        value = get_log_dirpath();
    }
    let mut log_dirpath = std::path::PathBuf::from(value);
    let hostname = gethostname()
        .into_string()
        .unwrap_or(String::from("unknown_hostname"));
    let log_node_dirpath = log_dirpath.join(std::path::PathBuf::from(hostname.clone()));
    
    if ! log_node_dirpath.exists() {
        match create_dir_all(&log_dirpath) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }
    if plugin.state.uid.is_some() && ( std::fs::metadata(&log_node_dirpath).unwrap().uid() != plugin.state.uid.unwrap() ) {
        match chown(&log_node_dirpath, plugin.state.uid, plugin.state.gid) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }
    if plugin.state.gid.is_some() && ( std::fs::metadata(&log_node_dirpath).unwrap().gid() != plugin.state.gid.unwrap() ) {
        match chown(&log_node_dirpath, plugin.state.uid, plugin.state.gid) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }

    log_dirpath = match &plugin.state.jobid {
        Some(j) => {
            let dirname = format!("job_{}", j);
            let ld = log_node_dirpath.join(std::path::PathBuf::from(dirname));
            if ! ld.exists() {
                match create_dir_all(&ld) {
                    Ok(_) => (),
                    Err(_) => {
                        return;
                    },
                }
            }
            if plugin.state.uid.is_some() && ( std::fs::metadata(&ld).unwrap().uid() != plugin.state.uid.unwrap() ) {
                match chown(&ld, plugin.state.uid, plugin.state.gid) {
                    Ok(_) => (),
                    Err(_) => {
                        return;
                    },
                }
            }
            if plugin.state.gid.is_some() && ( std::fs::metadata(&ld).unwrap().gid() != plugin.state.gid.unwrap() ) {
                match chown(&ld, plugin.state.uid, plugin.state.gid) {
                    Ok(_) => (),
                    Err(_) => {
                        return;
                    },
                }
            }
            ld
        },
        None => log_node_dirpath,
    };
    let log_filepath = log_dirpath
        .join(std::path::PathBuf::from("stage0.log"));

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&log_filepath)
        .unwrap_or_else(|e| panic!("Error opening file: {:?} {}", log_filepath, e));

    if plugin.state.uid.is_some() && ( std::fs::metadata(&log_filepath).unwrap().uid() != plugin.state.uid.unwrap() ) {
        match chown(&log_filepath, plugin.state.uid, plugin.state.gid) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }
    if plugin.state.gid.is_some() && ( std::fs::metadata(&log_filepath).unwrap().gid() != plugin.state.gid.unwrap() ) {
        match chown(&log_filepath, plugin.state.uid, plugin.state.gid) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }
    stage0_log(log_dirpath, hostname, arg);
}

fn stage0_log(log_dirpath: PathBuf, hostname: String, arg: &str) {
    if ! log_dirpath.exists() {
        match create_dir_all(&log_dirpath) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }

    let log_filepath = log_dirpath
        .join(std::path::PathBuf::from("stage0.log"));

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&log_filepath)
        .unwrap_or_else(|e| panic!("Error opening file: {:?} {}", log_filepath, e));

    let tz_name = match iana_time_zone::get_timezone() {
        Ok(tz) => tz,
        Err(_) => {
            return;
        },
    };
    let local_tz: Tz = match tz_name.parse() {
        Ok(tz) => tz,
        Err(_) => {
            return;
        },
    };
    let now = Utc::now();
    let local_time = now.with_timezone(&local_tz); 
    let timestamp = local_time.format("%Y-%m-%dT%H:%M:%S%.3f");

    let msg = String::from(format!("[{}] {} - {} - {}", timestamp, hostname, get_versioned_plugin_name(), arg));

    let _ = writeln!(file, "{}", msg);
    let _ = file.flush();
    let _ = file.sync_all();
}
