use std::collections::HashMap;
use std::env::set_var;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use gethostname::gethostname;
use serde::{Deserialize, Serialize};
use tracing::{info};

use slurm_spank::{Context, SpankHandle};
use raster::{expand_vars_string};

use crate::{CACHE_PATH, LOCAL2REMOTE_VARNAME, LOCAL2REMOTE_FILENAME, create_dir_path, SpankStage0, get_log_dirpath, remote_log, spank_getenv};

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
    pub(crate) caller_id: Option<String>,
    pub(crate) job_arg: Vec<String>,
    pub(crate) job_env: HashMap<String, String>,
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
            caller_id: None,
            job_arg: vec![],
            job_env: HashMap::from([]),
        }
    }
}

pub(crate) fn get_local_task_id(state: &Stage0State) -> u32 {
    match state.job_env.get("SLURM_LOCALID") {
        None => {
            return u32::MAX
        },
        Some(id) => match id.parse::<u32>() {
            Ok(num) => {
                return num;
            },
            Err(_) => {
                return u32::MAX;
            },
        },
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
    info!("STAGE0_LOGDIR: {}", logdir);

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
    info!("STAGE0_USERNAME: {}", username);

    key = "SLURM_STAGE0_CALLER_ID";
    let pid = std::process::id();
    let hostname = gethostname()
        .into_string()
        .unwrap_or(String::from("unknown_hostname"));

    value = format!("{hostname}_pid_{pid}");
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }
    plugin.state.caller_id = Some(value);

    let caller_id = match &plugin.state.caller_id {
        Some(u) => u,
        None => &String::from("None"),
    };
    info!("STAGE0_CALLER_ID: {}", caller_id);

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
    // We need the JobID to build the folder name.
    remote_log(plugin, spank, &format!("STAGE0_LOGDIR: {}", value));
    //Moved here because we need to know the folder to log in
    remote_log(plugin, spank, &format!("STAGE0_JOBID: {}", &plugin.state.jobid.clone().unwrap()));

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

    key = "SLURM_STAGE0_CALLER_ID";
    let hostname = gethostname()
        .into_string()
        .unwrap_or(String::from("unknown_hostname"));

    let mut value = format!("{hostname}");

    match spank.job_id() {
        Ok(id) => {
            value = format!("{value}_job_{id}");
        },
        Err(_) => {
            value = format!("{value}_job_unknown");
        },
    }

    match spank.job_stepid() {
        Ok(id) => {
            value = format!("{value}_step_{id}");
        },
        Err(_) => {},
    }

    match spank.task_id() {
        Ok(id) => {
            value = format!("{value}_task_{id}");
        },
        Err(_) => {},
    }

    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }
    plugin.state.caller_id = Some(value);

    let caller_id = match &plugin.state.caller_id {
        Some(u) => u,
        None => &String::from("None"),
    };
    remote_log(plugin, spank, &format!("STAGE0_CALLER_ID: {}", caller_id));

    key = LOCAL2REMOTE_VARNAME;
    value = spank_getenv(spank, key).to_string();
    match value.as_str() {
        "None" => {
            unsafe {std::env::remove_var(key);}
        },
        _ => {
            unsafe {std::env::set_var(key, OsStr::new(&value));}
        },
    }
    remote_log(plugin, spank, &format!("{}: {}", key, value));

    // Translate job env
    remote_log(plugin, spank, &format!("STIKKAZZI"));
    let vc = get_job_arg(spank);
    plugin.state.job_arg = vc.clone();
    let hm = get_job_env(spank);
    plugin.state.job_env = hm.clone();
    
    // Write input.json
    jobarg2cache(plugin, spank); 
    jobenv2cache(plugin, spank); 
    remote_log(plugin, spank, &format!("AMMAZZI"));
    
    let mut i = 0;
    for a in vc.clone() {
        remote_log(plugin, spank, &format!("JOBARG MEMBER: {i} = {a}"));
        i = i + 1;
    }
    
    for (k,v) in hm.clone() {
        remote_log(plugin, spank, &format!("JOBENV VARIABLE: {k} = {v}"));
    }

    Ok(())
}

pub(crate) fn get_job_arg(spank: &mut SpankHandle) -> Vec<String> {
    let vec = match spank.job_argv() {
        Ok(v) => v,
        Err(_) => [].to_vec(),
    };

    let mut vc: Vec<String> = vec![];
    for v in vec.into_iter() {
        let value = v.to_string();
        vc.push(value);
    }

    return vc;
}

pub(crate) fn get_job_env(spank: &mut SpankHandle) -> HashMap<String,String> {
    let vec = match spank.job_env() {
        Ok(v) => v.clone(),
        Err(_) => [].to_vec(),
    };
    let mut h: HashMap<String,String> = HashMap::from([]);
    for v in vec.into_iter() {
        let (key, value) = v.split_once("=").unwrap();
        h.insert(key.to_string(), value.to_string());
    };

    // Integrate if needed
    if h.get("SLURM_NODEID").is_none() {
        let nodeid = match spank.job_nodeid() {
            Ok(nid) => Some(nid),
            Err(_) => None,
        };
        if nodeid.is_some() {
            let nodestr = nodeid.unwrap().to_string();
            h.insert("SLURM_NODEID".to_string(), nodestr); 
        }
    }

    return h;
}

pub(crate) fn get_cache_dir_path(plugin: &mut SpankStage0) -> String {
    let jobenv = plugin.state.job_env.clone();
    let opt_jobenv;
    if jobenv.len() == 0 {
        opt_jobenv = None;
    } else {
        opt_jobenv = Some(jobenv);
    }

    let cache_path = expand_vars_string(CACHE_PATH.to_string(), &opt_jobenv).unwrap();

    // Collect CALLER_ID
    let caller_id = match &plugin.state.caller_id {
        Some(u) => u,
        None => {
            return String::from("");
        },
    };

    let cache_dirname = caller_id;
    let cache_dir_path = format!("{cache_path}/{cache_dirname}");
    return cache_dir_path;
}

pub(crate) fn set_local2remote_env_var(plugin: &mut SpankStage0) {
    // Check file existence
    let cache_dir_path = get_cache_dir_path(plugin);
    let file_path_str = format!("{cache_dir_path}/{LOCAL2REMOTE_FILENAME}");
    let file_path = Path::new(&file_path_str);
    info!("HERE: {}", file_path_str);

    if ! file_path.exists() {
        return;
    };
    info!("THERE");

    let content = match read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => {
            return;
        },
    };
    info!("MORE:\n{}", content);

    unsafe {
        set_var(LOCAL2REMOTE_VARNAME, OsStr::new(&content));
    }
    info!("MORE THAN EVER");
}

pub(crate) fn jobenv2cache(plugin: &mut SpankStage0, spank: &mut SpankHandle) {

    remote_log(plugin, spank, &format!("STIKKAZZI"));
    let cache_dir_path = get_cache_dir_path(plugin);
    let _ = create_dir_path(plugin, Path::new(&cache_dir_path));
    let cache_file_path = format!("{cache_dir_path}/jobenv.json");
    remote_log(plugin, spank, &format!("FILE: {}", cache_file_path));

    let content = match serde_json::to_string(&plugin.state.job_env) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot serialize jobenv to json");
        }
    };

    let mut file = match File::create(&cache_file_path) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot open {cache_file_path}");
        }
    };

    let _ = match file.write_all(content.as_bytes()) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot write to {cache_file_path}");
        }
    };
    let _ = file.flush();
    let _ = file.sync_all();
    remote_log(plugin, spank, &format!("BIGAZZI"));
}

pub(crate) fn jobarg2cache(plugin: &mut SpankStage0, _spank: &mut SpankHandle) {

    let cache_dir_path = get_cache_dir_path(plugin);
    let _ = create_dir_path(plugin, Path::new(&cache_dir_path));
    let cache_file_path = format!("{cache_dir_path}/jobarg.json");

    let content = match serde_json::to_string(&plugin.state.job_arg) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot serialize jobarg to json");
        }
    };

    let mut file = match File::create(&cache_file_path) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot open {cache_file_path}");
        }
    };

    let _ = match file.write_all(content.as_bytes()) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot write to {cache_file_path}");
        }
    };
    let _ = file.flush();
    let _ = file.sync_all();
}
