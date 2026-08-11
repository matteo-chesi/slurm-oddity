use std::env;
use std::error::Error;
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;
use clap::{Parser, ValueEnum};
use whoami::username;

pub mod autoupdate;
pub mod config;
pub mod dispatch;
pub mod log;
pub mod srun;

use raster::{Config};

use crate::autoupdate::{AutoUpdate, auto_update, get_requested_exe_path};
use crate::log::log;
use crate::config::load_config;
use crate::dispatch::dispatch_execution;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const COMMAND_NAME: &str = "stage1";
pub(crate) const CACHE_PATH: &str = "${HOME}/.local/share/cosmodrome/cache";

#[derive(Parser, Clone, Debug)]
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

#[derive(Clone, Copy, ValueEnum, Debug)]
enum Context {
    Local,
    Allocator,
    Remote,
}

#[derive(Clone, Copy, ValueEnum, Debug, PartialEq)]
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

#[derive(Debug)]
struct State {
    exe_user: String,
    exe_path: String,
    exe_args: String,
    args: Args,
    config: Config,
}

pub(crate) fn get_versioned_command_name() -> String {
    return String::from(format!("{}-v{}", COMMAND_NAME, VERSION));
}

fn run(args: &Args) {
    let state = load_state(args);
    //log(&format!("STATE:\n{:#?}", state));

    let requested_exe_path = get_requested_exe_path(&state);
    if state.exe_path != requested_exe_path {
        Command::new(&requested_exe_path)
        .args(&env::args().collect::<Vec<String>>()[1..])
        .output()
        .expect(&format!("failed to execute {}", requested_exe_path));
        return;
    }

    log_start(&state);
    dispatch_execution(&state);
    log_end(&state);
}

fn main() {
    let args = Args::parse();
    run(&args);
}

pub(crate) fn load_state(args: &Args) -> State {
    let state_args = args.clone();

    let exe_user = username().unwrap_or(String::from("unknown_user"));
    
    let abs_exe_path = std::path::absolute(std::path::PathBuf::from(&env::args()
        .collect::<Vec<String>>()[0]))
        .unwrap();
    let exe_path = abs_exe_path.display().to_string();

    let exe_args = env::args().collect::<Vec<String>>()[1..].join(" ");
    
    let state = State {
        exe_user: exe_user,
        exe_path: exe_path,
        exe_args: exe_args,
        args: state_args,
        config: load_config(),
    };

    state
}

fn log_start(state: &State) {
    log(&format!("Executing as \"{}\" : {} {}",
            state.exe_user,
            state.exe_path,
            state.exe_args));
}

fn log_end(state: &State) {
    if state.args.payload.is_some() {
        log(&format!("Executed  as \"{}\" : Context {:#?} - Function {:#?} - Payload {}", 
                state.exe_user, 
                state.args.context,
                state.args.function,
                state.args.payload.clone().unwrap()));

    } else {
        log(&format!("Executed  as \"{}\" : Context {:#?} - Function {:#?}",
                state.exe_user,
                state.args.context,
                state.args.function));
    }
}

pub(crate) fn expand_vars_string(s: String) -> Result<String, Box<dyn Error>> {
    match shellexpand::env(&s) {
        Ok(ok) => return Ok(ok.to_string()),
        Err(_) => return Ok(s),
    };
}

pub(crate) fn create_dir_path(dir_path: &Path) -> Result<(), Box<dyn Error>> {
    let path = dir_path;

    if ! path.exists() {

        match path.parent() {
            Some(pp) => {
                if ! pp.exists() {
                    create_dir_path(pp)?;
                }
            },
            None => {},
        };
        create_dir_all(path)?;
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
