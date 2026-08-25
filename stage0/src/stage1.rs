use std::env::{current_dir, set_current_dir, set_var};
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Output};
use users::get_current_uid;

use slurm_spank::{Context, SpankHandle};

use crate::{SpankStage0, log, remote_log, set_local2remote_env_var, spank_getenv};

pub(crate) fn run_stage1(
    stage0: &mut SpankStage0,
    spank: &mut SpankHandle,
    context: String,
    function: String,
    payload: Option<String>)
-> Result<Output, Box<dyn Error>> {

    let result = match spank.context() {
        Ok(Context::Local) | Ok(Context::Allocator) | Ok(Context::Slurmd) => {
            local_run_stage1(stage0, spank, context, function, payload)
        },
        Ok(Context::Remote) => {
            remote_run_stage1(stage0, spank, context, function, payload)
        }
        _ => {
            Err("Nothing to run".into())
        },
    };

    return result;
}

fn local_run_stage1(
    stage0: &mut SpankStage0,
    _spank: &mut SpankHandle,
    context: String,
    function: String,
    payload: Option<String>)
-> Result<Output, Box<dyn Error>> {

    let mut fstr = format!("{}({},{},{})", "run_stage1", context, function, "None");
    let mut pl;
    let cur_uid = get_current_uid();
    if payload.is_some() {
        pl = payload.clone().unwrap();
        fstr = format!("{}({},{},{})", "run_stage1", context, function, pl.as_str())
    }
    log(&format!("[UID: {}] Calling: {}", &cur_uid, &fstr));

    let cmdname;
    if cur_uid == 0 {
        return Err("Cannot run stage1 as root".into());
    } else {
        if ! Path::new(&stage0.config.stage1_user_path).exists() {
            if ! Path::new(&stage0.config.stage1_system_path).exists() {
                let msg = format!("ERROR: cannot find stage1 executable at \"{}\"", &stage0.config.stage1_system_path);
                log(&msg);
                return Err(msg.into());
            } else {
                cmdname = &stage0.config.stage1_system_path;
            }
        } else {
            cmdname = &stage0.config.stage1_user_path;
        }
    }

    let mut cmdargs = vec!["--context", &context, "--function", &function];
    let mut cmdstr = format!("{} --context {} --function {}", &cmdname, context, function);

    if payload.is_some() {
        pl = payload.clone().unwrap();
        cmdargs.push("--payload");
        cmdargs.push(pl.as_str());
        cmdstr = format!("{} --context {} --function {} --payload {}", &cmdname, context, function, pl.as_str());
    }

    log(&format!("Executing: {}", &cmdstr));

    let result = Command::new(cmdname)
        .args(cmdargs.clone())
        .output();

    if result.is_err() {
        return Err(format!("ERROR: Failed to spawn command: {} {:#?}", cmdname, cmdargs).into());
    };

    let output = result.unwrap();
    log(&format!("Executed : {}", &cmdstr));

    let exit_code = match output.status.code() {
        Some(rc) => rc.to_string(),
        None => String::from("killed by signal"),
    };
    log(&format!("RC: {}", exit_code));

    let mut stdout = String::from_utf8(output.stdout.clone()).unwrap_or(String::from(""));
    if ! stdout.is_empty() {
        stdout.pop();
        let lines = stdout.split('\n');
        for line in lines {
            log(&format!("stdout: {}", line));
        }
    };
    let mut stderr = String::from_utf8(output.stderr.clone()).unwrap_or(String::from(""));
    if ! stderr.is_empty() {
        stderr.pop();
        let lines = stderr.split('\n');
        for line in lines {
            log(&format!("stderr: {}", line));
        }
    };
    
    if function == "init_post_opt" {
        set_local2remote_env_var(stage0);
    }

    return Ok(output);
}

fn remote_run_stage1(
    stage0: &mut SpankStage0,
    spank: &mut SpankHandle,
    context: String,
    function: String,
    payload: Option<String>) 
-> Result<Output, Box<dyn Error>> {

    let mut fstr = format!("{}({},{},{})", "run_stage1", context, function, "None");
    let mut pl;
    let cur_uid = get_current_uid();
    if payload.is_some() {
        pl = payload.clone().unwrap();
        fstr = format!("{}({},{},{})", "run_stage1", context, function, pl.as_str())
    }
    remote_log(stage0, spank, &format!("[UID: {}] Calling: {}", &cur_uid, &fstr));

    let prev_dir = match current_dir() {
        Ok(d) => d,
        Err(_) => PathBuf::from("/"),
    };
    let cur_dir = spank_getenv(spank, "PWD");
    let _ = set_current_dir(cur_dir);
    
    if function == "task_init" {
        set_job_home_env_var(spank);
    }

    let cmdname;
    let cmd2run;
    let mut cmdargs = vec![];
    let mut cmdstr;
    let username;
    let innercmd;
    if cur_uid == 0 {
        if stage0.state.username.is_none() {
            return Err("Unknown username".into());
        }
        
        if ! Path::new(&stage0.config.stage1_user_path).exists() {
            if ! Path::new(&stage0.config.stage1_system_path).exists() {
                let msg = format!("ERROR: cannot find stage1 executable at \"{}\"", &stage0.config.stage1_system_path);
                remote_log(stage0, spank, &msg);
                return Err(msg.into());
            } else {
                cmd2run = stage0.config.stage1_system_path.clone();
            }
        } else {
            cmd2run = stage0.config.stage1_user_path.clone();
        }

        cmdname = String::from("/usr/bin/su");
        username = stage0.state.username.clone().unwrap();
        if payload.is_some() {
            pl = payload.clone().unwrap();
            innercmd = format!("{} --context {} --function {} --payload {}", &cmd2run, &context, &function, pl.as_str());
            cmdargs = vec![
                &username,
                "-c", &innercmd,
            ];
            cmdstr = format!("{} {} -c {} --context {} --function {} --payload {}", &cmdname, &username, &cmd2run ,&context, &function, pl.as_str());
        } else {
            innercmd = format!("{} --context {} --function {}", &cmd2run, &context, &function);
            cmdargs = vec![
                &username,
                "-c", &innercmd,
            ];
            cmdstr = format!("{} {} -c {} --context {} --function {}", &cmdname, &username, &cmd2run, &context, &function);
        }
    } else {
        if ! Path::new(&stage0.config.stage1_user_path).exists() {
            if ! Path::new(&stage0.config.stage1_system_path).exists() {
                let msg = format!("ERROR: cannot find stage1 executable at \"{}\"", &stage0.config.stage1_system_path);
                remote_log(stage0, spank, &msg);
                return Err(msg.into());
            } else {
                cmdname = stage0.config.stage1_system_path.clone();
            }
        } else {
            cmdname = stage0.config.stage1_user_path.clone();
        }
        cmdargs = vec!["--context", &context, "--function", &function];
        cmdstr = format!("{} --context {} --function {}", &cmdname, context, function);
        if payload.is_some() {
            pl = payload.clone().unwrap();
            cmdargs.push("--payload");
            cmdargs.push(pl.as_str());
            cmdstr = format!("{} --context {} --function {} --payload {}", &cmdname, context, function, pl.as_str());
        }
    }

    remote_log(stage0, spank, &format!("Executing: {}", &cmdstr));

    let result = Command::new(cmdname)
        .args(cmdargs)
        .output();

    if result.is_err() {
        return Err("failed to execute process".into());
    }
    
    let output = result.unwrap();
    remote_log(stage0, spank, &format!("Executed : {}", &cmdstr));

    let exit_code = match output.status.code() {
        Some(rc) => rc.to_string(),
        None => String::from("killed by signal"),
    };
    remote_log(stage0, spank, &format!("RC: {}", exit_code));

    let mut stdout = String::from_utf8(output.stdout.clone()).unwrap_or(String::from(""));
    if ! stdout.is_empty() {
        stdout.pop();
        let lines = stdout.split('\n');
        for line in lines {
            remote_log(stage0, spank, &format!("stdout: {}", line));
        }
    };
    let mut stderr = String::from_utf8(output.stderr.clone()).unwrap_or(String::from(""));
    if ! stderr.is_empty() {
        stderr.pop();
        let lines = stderr.split('\n');
        for line in lines {
            remote_log(stage0, spank, &format!("stderr: {}", line));
        }
    };
    
    let _ = set_current_dir(prev_dir);

    return Ok(output);
}

fn set_job_home_env_var(spank: &mut SpankHandle) {
    let home_dir = spank_getenv(spank, "HOME");
    if home_dir != "" {
        unsafe {
            set_var("HOME", OsStr::new(&home_dir));
        }
    }
}
