use std::env;
use clap::{Parser, ValueEnum};
use whoami::username;

pub mod log;

use crate::log::log;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const COMMAND_NAME: &str = "stage1";

#[derive(Parser)]
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

#[derive(Clone, Copy, ValueEnum, Debug)]
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

pub(crate) fn get_versioned_command_name() -> String {
    return String::from(format!("{}-v{}", COMMAND_NAME, VERSION));
}

fn run(args: &Args) {
    let exe_user = username().unwrap_or(String::from("unknown_user"));

    /*
    let exe_path = match env::current_exe() {
        Ok(exe_path) => exe_path.display().to_string(),
        Err(_) => String::from("unknown_binary_path"),
    };
    */
    let abs_exe_path = std::path::absolute(std::path::PathBuf::from(&env::args()
        .collect::<Vec<String>>()[0]))
        .unwrap();
    let exe_path = abs_exe_path.display();

    let exe_args = env::args().collect::<Vec<String>>()[1..].join(" ");
    log(&format!("Executing as \"{}\" : {} {}", exe_user, exe_path, exe_args));
    if args.payload.is_some() {
        log(&format!("Executed  as \"{}\" : Context {:#?} - Function {:#?} - Payload {}", exe_user, args.context, args.function, args.payload.clone().unwrap()));
    } else {
        log(&format!("Executed  as \"{}\" : Context {:#?} - Function {:#?}", exe_user, args.context, args.function));
    }
}

fn main() {
    let args = Args::parse();
    run(&args);
}


