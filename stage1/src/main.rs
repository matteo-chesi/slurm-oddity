use std::env;
use clap::{Parser, ValueEnum};
use whoami::username;

pub mod config;
pub mod dispatch;
pub mod log;
pub mod srun;

use raster::{Config};

use crate::log::log;
use crate::config::load_config;
use crate::dispatch::dispatch_execution;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const COMMAND_NAME: &str = "stage1";

#[derive(Parser, Clone)]
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
