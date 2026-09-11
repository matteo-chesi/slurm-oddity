use std::collections::HashMap;
//use std::env::{self, remove_var, set_var, var};
use std::env::{remove_var, set_var, var};
use std::error::Error;
use std::fs::{
    File,
    OpenOptions,
    create_dir_all,
};
//use std::io::Write;
use std::os::unix::fs::{chown, MetadataExt};
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{Local};
//use chrono::{Utc, Local};
//use chrono_tz::Tz;
//use gethostname::gethostname;
use tracing_subscriber::fmt;
use tracing::{Level};

use raster::expand_vars_string;

use crate::{
    DATETIME_FORMAT,
    LOG_PATH,
    LOCAL_LOG_FILENAME,
    NAME,
    REMOTE_LOG_FILENAME,
    Context,
    State,
};

/*
pub(crate) fn log(arg: &str) {
    let log_dirpath = std::path::PathBuf::from(get_log_dirpath());

    if ! log_dirpath.exists() {
        match create_dir_all(&log_dirpath) {
            Ok(_) => (),
            Err(_) => {
                return;
            },
        }
    }

    let log_filepath = log_dirpath
        .join(std::path::PathBuf::from("stage1.log"));

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

    let hostname = gethostname()
            .into_string()
            .unwrap_or(String::from("unknown_hostname"));

    let now = Utc::now();
    let local_time = now.with_timezone(&local_tz);
    let timestamp = local_time.format("%Y-%m-%dT%H:%M:%S%.3f");

    let msg = String::from(format!("{} - {}", get_versioned_command_name(), arg));
    let time_msg = String::from(format!("[{}] {} - {}", timestamp, hostname, msg));

    let _ = writeln!(file, "{}", time_msg);
    let _ = file.flush();
    let _ = file.sync_all();
    eprintln!("{msg}");
}

pub(crate) fn get_log_dirpath() -> String {
    let log_dirpath;

    let variable = match env::var("SLURM_STAGE0_LOGDIR") {
        Ok(dir) => Some(dir),
        Err(_) => None,
    };

    if variable.is_some() {
        log_dirpath = std::path::PathBuf::from(variable.unwrap());
    } else {
        let home_dir = env::home_dir().unwrap();
        log_dirpath = home_dir.join(std::path::PathBuf::from("slurm-oddity"));
    }
    
    let hostname = gethostname()
            .into_string()
            .unwrap_or(String::from("unknown_hostname"));

    let log_dirpath = log_dirpath.join(std::path::PathBuf::from(hostname));
    return log_dirpath.into_os_string().to_string_lossy().to_string();
}
*/

pub(crate) fn setup_tracing(state: &mut State) -> Result<(), Box<dyn Error>> {
    init_log_file(state);
    set_panic_hook();
    let path = &state.log_file;

    let file = match OpenOptions::new().create(false).append(true).open(path) {
        Ok(f) => f,
        Err(_) => return Ok(()),
    };

    let subscriber = fmt()
        .with_writer(Mutex::new(file))
        .with_ansi(false)
        .with_target(false)
        .with_level(false)
        .with_timer(fmt::time::LocalTime::rfc_3339())
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

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

fn log_dir(state: &mut State) -> PathBuf {
    let mut log_dir = PathBuf::from("/tmp".to_string());
    let mut jobenv = None;

    match state.args.context {
        Context::Remote => {
            jobenv = Some(state.job_env.clone());
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

fn create_file_path(state: &mut State, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut uid = None;
    let mut gid = None;

    match state.args.context {
        Context::Remote => {
            uid = match &state.job {
                Some(j) => Some(j.uid),
                None => None,
            };
            gid = match &state.job {
                Some(j) => Some(j.gid),
                None => None,
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

fn get_remote_environment_for_log_filename(state: &mut State) -> HashMap<String,String> {

    let mut jobenv = state.job_env.clone();
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


pub(crate) fn init_log_file(state: &mut State) {

    let path;
    match set_log_filename(state) {
        Ok(_) => {
            path = state.log_file.clone();
        },
        Err(_) => return,
    }

    if ! path.exists() {
        match create_file_path(state, &path) {
            Ok(_) => {},
            Err(_) => return,
        }
    }
}

pub(crate) fn remove_empty_log_file(state: &mut State) {
    let path = state.log_file.clone();
    let metadata = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => {
            return;
        },
    };

    if metadata.len() != 0 {
        return;
    };

    let _ = std::fs::remove_file(&path);

    // Remove empty directory tree
    let mut parent = match path.parent() {
        Some(dir) => dir,
        None => {
            return;
        },
    };

    let mut dirname = match parent.file_name() {
        Some(name) => name,
        None => {
            return;
        },
    };
    
    let mut done = false;
    let mut last = false;
    while ! done {

        if dirname == NAME {
            last = true;
        }

        let read_dir = match std::fs::read_dir(parent) {
            Ok(r) => r,
            Err(_) => {
                break;
            },
        };

        if read_dir.count() != 0 {
            break;
        }

        let _ = std::fs::remove_dir(parent);

        parent = match parent.parent() {
            Some(dir) => dir,
            None => {
                break;
            },
        };

        dirname = match parent.file_name() {
            Some(name) => name,
            None => {
                break;
            },
        };
        
        if last == true {
            done = true;
        }
    }

}

fn set_log_filename(state: &mut State) -> Result<(), Box<dyn Error>> {
    let log_dir = log_dir(state);
    let log_filename;

    match state.args.context {
        Context::Local => {
            set_local_environment_for_log_filename();
            log_filename = match expand_vars_string(LOCAL_LOG_FILENAME.to_string(), &None) {
                Ok(s) => PathBuf::from(s),
                Err(_) => return Err("Cannot set log filename for local context".into()),
            };
            unset_local_environment_for_log_filename();
        },
        Context::Remote => {
            let jobenv = get_remote_environment_for_log_filename(state);
            log_filename = match expand_vars_string(REMOTE_LOG_FILENAME.to_string(), &Some(jobenv)) {
                Ok(s) => PathBuf::from(s),
                Err(_) => return Err("Cannot set log filename for remote context".into()),
            };
        },
        _ => return Err("Cannot set log filename".into()),
   }
   state.log_file = log_dir.join(log_filename);

   Ok(())
}

