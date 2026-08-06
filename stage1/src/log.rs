use std::env;
use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::io::Write;

use chrono::Utc;
use chrono_tz::Tz;
use gethostname::gethostname;

use crate::get_versioned_command_name;


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
    println!("{msg}");
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
        log_dirpath = home_dir.join(std::path::PathBuf::from("cosmodrome"));
    }
    
    let hostname = gethostname()
            .into_string()
            .unwrap_or(String::from("unknown_hostname"));

    let log_dirpath = log_dirpath.join(std::path::PathBuf::from(hostname));
    return log_dirpath.into_os_string().to_string_lossy().to_string();
}
