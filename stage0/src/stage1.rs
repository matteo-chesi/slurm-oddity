use std::path::Path;
use std::process::Command;

use slurm_spank::{Context, SpankHandle};

use crate::{SpankStage0, log, remote_log};

pub(crate) fn run_stage1(stage0: &mut SpankStage0, spank: &mut SpankHandle, context: String, function: String, payload: Option<String>) {

    let _ = match spank.context() {
        Ok(Context::Local) | Ok(Context::Allocator) | Ok(Context::Slurmd) => {
            local_run_stage1(stage0, spank, context, function, payload);
        },
        Ok(Context::Remote) => {
            remote_run_stage1(stage0, spank, context, function, payload);
        }
        _ => {
                return ();
        },
    };
}

fn local_run_stage1(stage0: &mut SpankStage0, _spank: &mut SpankHandle, context: String, function: String, payload: Option<String>) {

    let mut fstr = format!("{}({},{},{})", "run_stage1", context, function, "None");
    let mut pl;
    if payload.is_some() {
        pl = payload.clone().unwrap();
        fstr = format!("{}({},{},{})", "run_stage1", context, function, pl.as_str())
    }
    log(&format!("Calling: {}", &fstr));

    let cmdname;
    if ! Path::new(&stage0.config.stage1_user_path).exists() {
        if ! Path::new(&stage0.config.stage1_system_path).exists() {
            log(format!("ERROR: cannot find stage1 executable at \"{}\"", &stage0.config.stage1_system_path).as_str());

            return;
        } else {
            cmdname = &stage0.config.stage1_system_path;
        }
    } else {
        cmdname = &stage0.config.stage1_user_path;
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

    let output = Command::new(cmdname)
        .args(cmdargs)
        .output()
        .expect("failed to execute process");

    log(&format!("Executed : {}", &cmdstr));

    let exit_code = match output.status.code() {
        Some(rc) => rc.to_string(),
        None => String::from("killed by signal"),
    };
    log(&format!("RC: {}", exit_code));

    let mut stdout = String::from_utf8(output.stdout).unwrap_or(String::from(""));
    if ! stdout.is_empty() {
        stdout.pop();
        let lines = stdout.split('\n');
        for line in lines {
            log(&format!("stdout: {}", line));
        }
    };
    let mut stderr = String::from_utf8(output.stderr).unwrap_or(String::from(""));
    if ! stderr.is_empty() {
        stderr.pop();
        let lines = stderr.split('\n');
        for line in lines {
            log(&format!("stderr: {}", line));
        }
    };
}

fn remote_run_stage1(stage0: &mut SpankStage0, spank: &mut SpankHandle, context: String, function: String, payload: Option<String>) {

    let mut fstr = format!("{}({},{},{})", "run_stage1", context, function, "None");
    let mut pl;
    if payload.is_some() {
        pl = payload.clone().unwrap();
        fstr = format!("{}({},{},{})", "run_stage1", context, function, pl.as_str())
    }
    remote_log(stage0, spank, &format!("Calling: {}", &fstr));

    let cmdname;
    if ! Path::new(&stage0.config.stage1_user_path).exists() {
        if ! Path::new(&stage0.config.stage1_system_path).exists() {
            remote_log(stage0, spank, format!("ERROR: cannot find stage1 executable at \"{}\"", &stage0.config.stage1_system_path).as_str());
            return;
        } else {
            cmdname = stage0.config.stage1_system_path.clone();
        }
    } else {
        cmdname = stage0.config.stage1_user_path.clone();
    }

    let mut cmdargs = vec!["--context", &context, "--function", &function];
    let mut cmdstr = format!("{} --context {} --function {}", &cmdname, context, function);

    if payload.is_some() {
        pl = payload.clone().unwrap();
        cmdargs.push("--payload");
        cmdargs.push(pl.as_str());
        cmdstr = format!("{} --context {} --function {} --payload {}", &cmdname, context, function, pl.as_str());
    }

    remote_log(stage0, spank, &format!("Executing: {}", &cmdstr));

    let output = Command::new(cmdname)
        .args(cmdargs)
        .output()
        .expect("failed to execute process");

    remote_log(stage0, spank, &format!("Executed : {}", &cmdstr));

    let exit_code = match output.status.code() {
        Some(rc) => rc.to_string(),
        None => String::from("killed by signal"),
    };
    remote_log(stage0, spank, &format!("RC: {}", exit_code));

    let mut stdout = String::from_utf8(output.stdout).unwrap_or(String::from(""));
    if ! stdout.is_empty() {
        stdout.pop();
        let lines = stdout.split('\n');
        for line in lines {
            remote_log(stage0, spank, &format!("stdout: {}", line));
        }
    };
    let mut stderr = String::from_utf8(output.stderr).unwrap_or(String::from(""));
    if ! stderr.is_empty() {
        stderr.pop();
        let lines = stderr.split('\n');
        for line in lines {
            remote_log(stage0, spank, &format!("stderr: {}", line));
        }
    };
}
