use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::create_dir_all;
use std::fs::Permissions;
//use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use const_format::formatcp;
use nix::libc::{gid_t, uid_t};
use clap::{Parser, ValueEnum};
use serde::{Serialize};
use tracing::{error, info};
use whoami::username;

pub mod autoupdate;
pub mod config;
pub mod dispatch;
pub mod edf;
pub mod iodata;
pub mod jobarg;
pub mod jobenv;
pub mod log;
pub mod podman;
pub mod slurmstepd;
pub mod srun;
pub mod sync;

use raster::{Config, EDF};

use crate::autoupdate::{AutoUpdate, auto_update, get_requested_exe_path};
use crate::edf::{local_load_edf, modify_edf_for_sbatch, remote_load_edf};
use crate::log::{setup_tracing};
use crate::config::{load_config, render_user_job_config, setup_imagestore};
use crate::dispatch::dispatch_execution;
use crate::iodata::{
    IOData,
    DataContainer,
    DataForward,
    get_iodata_from_stdin,
    send_iodata_to_stdout,
};
use crate::jobarg::{load_jobarg, load_jobarg_from_data};
use crate::jobenv::{load_jobenv, load_jobenv_from_data};
use crate::podman::{
    PODMAN_PIDFILE_NAME,
    podman_get_pid_from_file,
    podman_pull,
    podman_start,
    podman_stop,
};
use crate::sync::{
    sync_podman_pull,
    sync_podman_start,
    sync_podman_stop,
    sync_cleanup_fs_shared,
};

pub(crate) const COLOR: &str = "BLUE";
pub(crate) const NAME: &str = "slurm-oddity";
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const COMMAND_NAME: &str = "stage1";
pub(crate) const APP_NAME: &str = formatcp!("{}-{}", COMMAND_NAME, VERSION);
pub(crate) const LOG_PATH: &str = formatcp!("${{HOME}}/.local/share/{}/log", NAME);
pub(crate) const DATETIME_FORMAT: &str = "%Y%m%d";
pub(crate) const LOCAL_LOG_FILENAME: &str = formatcp!("${{DATETIME}}_${{CLUSTER_NAME}}/local_${{HOSTNAME}}_{}.log", COMMAND_NAME);
pub(crate) const REMOTE_LOG_FILENAME: &str = formatcp!("${{DATETIME}}_${{CLUSTER_NAME}}/job_${{SLURM_JOB_ID}}/${{HOSTNAME}}_{}.log", COMMAND_NAME);
pub(crate) const CACHE_PATH: &str = formatcp!("${{HOME}}/.local/share/{}/cache", NAME);
pub(crate) const LOCAL2REMOTE_VARNAME: &str = "SLURM_STAGE0_LOCAL2REMOTE_DATA";
pub(crate) const SLURM_BATCH_SCRIPT: u32 = 0xfffffffb;

#[derive(Serialize, Parser, Clone, Debug)]
#[command(about = "\n
Cosmodrome project stage1
The command should be called by slurm spank plugin stage0
The command should verify that all the calls works correctly and output is logged.", long_about = None)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    context: Context,
    
    #[arg(long)]
    function: Function,

    #[arg(long)]
    payload: Option<String>,
}

#[derive(Serialize, Clone, Copy, ValueEnum, Debug, PartialEq)]
enum Context {
    Local,
    Allocator,
    Remote,
}

#[derive(Serialize, Clone, Copy, ValueEnum, Debug, PartialEq)]
#[value(rename_all = "snake_case")]
enum Function {
    Init,
    InitPostOpt,
    UserInit,
    TaskInit,
    TaskInitPrivileged,
    TaskExit,
    Exit,
}

#[derive(Serialize, Debug)]
struct State {
    log_file: PathBuf,
    exe_user: String,
    exe_path: String,
    exe_args: String,
    args: Args,
    config: Config,
    job_env: HashMap<String, String>,
    job_arg: Vec<String>,
    edf: Option<EDF>,
    job: Option<Job>,
    run: Option<Run>,
}

#[derive(Clone, Serialize, Default, Debug)]
pub(crate) struct Job {
    uid: uid_t,
    gid: gid_t,
    jobid: u32,
    stepid: u32,
    local_task_id: u32,
    global_task_id: u32,
    nodeid: u32,
    local_task_count: u32,
    total_task_count: u32,
    cwd: String,
    euid: uid_t,
    egid: gid_t,
}

#[derive(Clone, Serialize, Default, Debug)]
pub(crate) struct Run {
    name: String,
    pid: usize,
    podman_tmp_path: String,
    syncfile_path: String,
}

#[derive(Clone, Serialize, Default, Debug)]
pub(crate) struct ConsoleOutput {
    console_out: String,
}

fn run(args: &Args) {
    
    let mut data = match get_iodata_from_stdin() {
        Ok(d) => d,
        Err(e) => {
            error!("Error: cannot read input data: {e}");
            return;
        },
    };
    let config = load_config(&mut data);
    let job_arg = load_jobarg_from_data(&mut data);
    let job_env = load_jobenv_from_data(&mut data);

    //info!("INPUT:\n{:#?}", data);

    let mut state = load_state(args, &config, &job_arg, &job_env);
    // Start Tracing
    let span = tracing::span!(tracing::Level::INFO, APP_NAME);
    let _ = span.enter();

    let requested_exe_path = get_requested_exe_path(&state);
    if state.exe_path != requested_exe_path {
        Command::new(&requested_exe_path)
        .args(&env::args().collect::<Vec<String>>()[1..])
        .output()
        .expect(&format!("failed to execute {}", requested_exe_path));
        return;
    }

    log_start(&state);
    dispatch_execution(&mut state, &mut data);
    log_end(&state);
}

fn main() {
    let args = Args::parse();
    run(&args);
}

pub(crate) fn load_state(
    args: &Args,
    config: &Config,
    job_arg: &Vec<String>,
    job_env: &HashMap<String, String>,
) -> State {
    let state_args = args.clone();
    let state_config = config.clone();
    let state_job_arg = job_arg.clone();
    let state_job_env = job_env.clone();

    let exe_user = username().unwrap_or(String::from("unknown_user"));
    
    let abs_exe_path = std::path::absolute(std::path::PathBuf::from(&env::args()
        .collect::<Vec<String>>()[0]))
        .unwrap();
    let exe_path = abs_exe_path.display().to_string();

    let exe_args = env::args().collect::<Vec<String>>()[1..].join(" ");
    
    let mut state = State {
        log_file: PathBuf::new(), 
        exe_user: exe_user,
        exe_path: exe_path,
        exe_args: exe_args,
        args: state_args,
        job_arg: state_job_arg,
        job_env: state_job_env,
        //job_arg: load_jobarg(),
        //job_env: load_jobenv(),
        config: state_config,
        edf: None,
        job: None,
        run: None,
    };

    let _ = setup_tracing(&mut state);

    state
}

fn log_start(state: &State) {
    info!("Executing as \"{}\" : {} {}",
            state.exe_user,
            state.exe_path,
            state.exe_args);
}

fn log_end(state: &State) {
    if state.args.payload.is_some() {
        info!("Executed  as \"{}\" : Context {:#?} - Function {:#?} - Payload {}", 
                state.exe_user, 
                state.args.context,
                state.args.function,
                state.args.payload.clone().unwrap());

    } else {
        info!("Executed  as \"{}\" : Context {:#?} - Function {:#?}",
                state.exe_user,
                state.args.context,
                state.args.function);
    }
}

pub(crate) fn expand_vars_string(s: String) -> Result<String, Box<dyn Error>> {
    match shellexpand::env(&s) {
        Ok(ok) => return Ok(ok.to_string()),
        Err(_) => return Ok(s),
    };
}

pub(crate) fn create_dir_path(dir_path: &Path, mode: u32) -> Result<(), Box<dyn Error>> {
    let path = dir_path;

    if ! path.exists() {

        match path.parent() {
            Some(pp) => {
                if ! pp.exists() {
                    create_dir_path(pp, 0o700)?;
                }
            },
            None => {},
        };
        create_dir_all(path)?;
        let perms = Permissions::from_mode(mode);
        std::fs::set_permissions(&path, perms)?;
    }
    Ok(())
}

pub(crate) fn get_cache_dir_path() -> String {
    let cache_path = expand_vars_string(CACHE_PATH.to_string()).unwrap();

    // Collect CALLER_ID
    let key = "SLURM_STAGE0_CALLER_ID";
    let caller_id = match std::env::var(key) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot read SLURM_STAGE0_CALLER_ID environment variable");
        },
    };

    let cache_dirname = caller_id;
    let cache_dir_path = format!("{cache_path}/{cache_dirname}");
    return cache_dir_path;
}

pub(crate) fn setup_folders(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let base_path = match state.run.clone() {
        Some(r) => r.podman_tmp_path,
        None => {
            let msg = "Error: couldn't find podman_tmp_path";
            info!(msg);
            return Err(msg.into());
        }
    };

    let dir_mode = 0o700;
    let mut dir_path;

    dir_path = format!("{}", base_path);
    create_dir_path(Path::new(&dir_path), dir_mode)?;

    dir_path = format!("{}/graphroot", base_path);
    create_dir_path(Path::new(&dir_path), dir_mode)?;

    dir_path = format!("{}/runroot", base_path);
    create_dir_path(Path::new(&dir_path), dir_mode)?;

    Ok(())
}

pub(crate) fn get_local_task_id(state: &State) -> u32 {
    return state.job.clone().unwrap().local_task_id;
}
/*
pub(crate) fn send_output(state: &State) {
    let json_string = match serde_json::to_string(state) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot serialize State to json");
        }
    };
    println!("{json_string}");
    let _ = io::stdout().flush();
}
*/

pub(crate) fn console_output(msg: &str) {
    let console_out = ConsoleOutput {
        console_out: msg.to_string(),
    };

    let console_out_json = match serde_json::to_string(&console_out) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot serialize console out message to json string");
        }
    };

    println!("{console_out_json}");
}

pub(crate) fn cleanup_fs_local(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    
    let base_path = match &state.run {
        Some(r) => &r.podman_tmp_path,
        None => {
            return Err("couldn't find run".into());
        }
    };

    while !Path::new(&base_path).exists() {
        info!("couldn't find {}, wait 1 sec and retry", &base_path);

        let pause = std::time::Duration::new(1, 0);
        std::thread::sleep(pause);
    }

    info!("delete {}", &base_path);
    match std::fs::remove_dir_all(&base_path) {
        Ok(_) => (),
        Err(e) => {
            let msg = format!("couldn't cleanup \"{:#?}\", error {}", &base_path, e);
            return Err(msg.into());
        }
    };
    Ok(())
}
