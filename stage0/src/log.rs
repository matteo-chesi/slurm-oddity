use std::collections::HashMap;
use std::env::{home_dir, remove_var, set_var, var};
use std::error::Error;
use std::fs::{
    File,
    //OpenOptions,
    create_dir_all,
};
//use std::io::Write;
use std::os::unix::fs::{chown, MetadataExt};
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use chrono::{Local};
//use chrono::{Utc, Local};
//use chrono_tz::Tz;
//use gethostname::gethostname;

use slurm_spank::{Context, SpankHandle};
use raster::expand_vars_string;

use crate::{
    //APP_NAME,
    DATETIME_FORMAT,
    LOG_PATH,
    LOCAL_LOG_FILENAME,
    REMOTE_LOG_FILENAME,
    SpankStage0,
    get_job_env
};
/*
pub(crate) fn log(arg: &str) {

        let log_dirpath = std::path::PathBuf::from(get_log_dirpath());
        let hostname = gethostname()
            .into_string()
            .unwrap_or(String::from("unknown_hostname"));
        let log_node_dirpath = log_dirpath.join(std::path::PathBuf::from(hostname.clone()));
        stage0_log(log_node_dirpath, hostname, arg);
}
*/

pub(crate) fn get_log_dirpath() -> String {
    let home_dir = home_dir().unwrap();
    let log_dirpath = home_dir.join(std::path::PathBuf::from("cosmodrome"));
    return log_dirpath.into_os_string().to_string_lossy().to_string();
}

/*
pub(crate) fn remote_log(plugin: &mut SpankStage0, _spank: &mut SpankHandle, arg: &str) {

    let value = match &plugin.state.stage0_logdir_path {
        Some(l) => l,
        None => &get_log_dirpath(),
    };

    /*
    let key = "SLURM_STAGE0_LOGDIR";
    let mut value = spank_getenv(spank, key);
    if value.is_empty() {
        value = get_log_dirpath();
    }
    */

    let log_dirpath = std::path::PathBuf::from(value);
    let hostname = gethostname()
        .into_string()
        .unwrap_or(String::from("unknown_hostname"));
    let log_node_dirpath = log_dirpath.join(std::path::PathBuf::from(hostname.clone()));
    
    if ! log_node_dirpath.parent().unwrap().exists() {
        match create_dir_all(&log_node_dirpath.parent().unwrap()) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }
    if plugin.state.uid.is_some() && ( std::fs::metadata(&log_node_dirpath.parent().unwrap()).unwrap().uid() != plugin.state.uid.unwrap() ) {
        match chown(&log_node_dirpath.parent().unwrap(), plugin.state.uid, plugin.state.gid) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }
    if plugin.state.gid.is_some() && ( std::fs::metadata(&log_node_dirpath.parent().unwrap()).unwrap().gid() != plugin.state.gid.unwrap() ) {
        match chown(&log_node_dirpath.parent().unwrap(), plugin.state.uid, plugin.state.gid) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }
    
    if ! log_node_dirpath.exists() {
        match create_dir_all(&log_node_dirpath) {
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

    /*
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
    */
    let log_filepath = log_node_dirpath
        .join(std::path::PathBuf::from("stage0.log"));


    let _file = OpenOptions::new()
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
    //stage0_log(log_dirpath, hostname, arg);
    stage0_log(log_node_dirpath, hostname, arg);
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

    let msg = String::from(format!("[{}] {} - {} - {}", timestamp, hostname, APP_NAME, arg));

    let _ = writeln!(file, "{}", msg);
    let _ = file.flush();
    let _ = file.sync_all();
}
*/
pub(crate) fn set_panic_hook() {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        panic_hook(panic_info);
        prev_hook(panic_info);
    }));
}

pub fn panic_hook(panic_info: &PanicHookInfo<'_>) {

    let payload_str;
    let location_str;
    if let Some(location) = panic_info.location() {
        location_str=format!("at {}:{}:{}",
            location.file(),
            location.line(),
            location.column(),
        );
    } else {
        location_str = format!("(unknown panic code location)");
    }

    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        payload_str = format!("PANIC: {s:?}");
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        payload_str = format!("PANIC: {s:?}");
    } else {
        payload_str = format!("PANIC");
    }

    tracing::error!("{} {}", payload_str, location_str);
}

pub(crate) fn format_error_chain(error: &dyn Error) -> String {
    let mut report = error.to_string();
    let mut current = error;

    while let Some(source) = current.source() {
        report.push_str(": ");
        report.push_str(&source.to_string());
        current = source;
    }

    report
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ErrorDestination {
    User,
    Tracing,
}

pub(crate) fn error_destination(context: Context) -> ErrorDestination {
    match context {
        Context::Local | Context::Allocator | Context::Remote => ErrorDestination::User,
        Context::Slurmd | Context::JobScript => ErrorDestination::Tracing,
    }
}

fn log_dir(spank: &mut SpankHandle) -> PathBuf {
    let mut log_dir = PathBuf::from("/tmp".to_string());
    let mut jobenv = None;

    match spank.context().unwrap() {
        Context::Remote => {
            jobenv = Some(get_job_env(spank));
        },
        _ => {
        },
    }
    match expand_vars_string(LOG_PATH.to_string(), &jobenv) {
        Ok(dp) => { log_dir = PathBuf::from(dp); },
        Err(_) => {},
    };

    return log_dir;
}

fn set_ownership(path: &Path, uid: Option<u32>, gid: Option<u32>) -> Result<(), Box<dyn Error>> {
    if (uid.is_some() &&
        ( std::fs::metadata(&path).unwrap().uid() != uid.unwrap())) ||
        ( gid.is_some() &&
        ( std::fs::metadata(&path).unwrap().gid() != gid.unwrap())) {
            match chown(&path, uid, gid) {
                Ok(_) => (),
                Err(_) => {
                    return Ok(());
                },
            }
    }
    Ok(())
}

fn create_dir_path(path: &Path, uid: Option<u32>, gid: Option<u32>) -> Result<(), Box<dyn Error>> {
    if ! path.exists() {
        match path.parent() {
            Some(pp) => {
                if ! pp.exists() {
                    create_dir_path(pp, uid, gid)?;
                }
            },
            None => {},
        };
        create_dir_all(path)?;
        set_ownership(path, uid, gid)?;
    }
    Ok(())
}

fn create_file_path(spank: &mut SpankHandle, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut uid = None;
    let mut gid = None;

    match spank.context().unwrap() {
        Context::Remote => {
            uid = match spank.job_uid() {
                Ok(uid) => Some(uid),
                Err(_) => None,
            };
            gid = match spank.job_gid() {
                Ok(gid) => Some(gid),
                Err(_) => None,
            };
        },
        _ => {},
    }

    if ! path.exists() {
        match path.parent() {
            Some(pp) => {
                if ! pp.exists() {
                    create_dir_path(pp, uid, gid)?;
                }
            },
            None => {},
        }
        File::create(path)?;
        set_ownership(path, uid, gid)?;
    }
    Ok(())
}

fn set_local_environment_for_log_filename() {
    let datetime = Local::now().format(&DATETIME_FORMAT).to_string();

    unsafe {
        set_var("DATETIME", datetime);
    };
    match var("CLUSTER_NAME") {
        Ok(_) => {},
        Err(_) => {
            unsafe {
                set_var("CLUSTER_NAME", "unknown");
            }
        },
    }
}

fn get_remote_environment_for_log_filename(spank: &mut SpankHandle) -> HashMap<String,String> {

    let mut jobenv = get_job_env(spank);
    let datetime = Local::now().format(&DATETIME_FORMAT).to_string();

    jobenv.insert("DATETIME".to_string(), datetime);

    match jobenv.get("CLUSTER_NAME") {
        Some(_) => {},
        None => {
            jobenv.insert("CLUSTER_NAME".to_string(), "unknown".to_string());
        },
    }

    return jobenv;
}


fn unset_local_environment_for_log_filename() {
    unsafe {
        remove_var("DATETIME");
    };
}

fn set_log_filename(plugin: &mut SpankStage0, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
    let log_dir = log_dir(spank);
    let log_filename;

    match spank.context().unwrap() {
        Context::Local => {
            set_local_environment_for_log_filename();
            log_filename = match expand_vars_string(LOCAL_LOG_FILENAME.to_string(), &None) {
                Ok(s) => PathBuf::from(s),
                Err(_) => return Err("Cannot set log filename for local context".into()),
            };
            unset_local_environment_for_log_filename();
        },
        Context::Remote => {
            let jobenv = get_remote_environment_for_log_filename(spank);
            log_filename = match expand_vars_string(REMOTE_LOG_FILENAME.to_string(), &Some(jobenv)) {
                Ok(s) => PathBuf::from(s),
                Err(_) => return Err("Cannot set log filename for remote context".into()),
            };
        },
        _ => return Err("Cannot set log filename".into()),
   }
   plugin.log_file = log_dir.join(log_filename);

   Ok(())
}

pub(crate) fn init_log_file(plugin: &mut SpankStage0, spank: &mut SpankHandle) {

    let path;
    match set_log_filename(plugin, spank) {
        Ok(_) => {
            path = plugin.log_file.clone();
        },
        Err(_) => return,
    }

    if ! path.exists() {
        match create_file_path(spank, &path) {
            Ok(_) => {},
            Err(_) => return,
        }
    }
}
