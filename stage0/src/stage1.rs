use std::env::{current_dir, set_current_dir, set_var};
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Read, Write};
//use std::process::Stdio;
//use tokio::runtime::Runtime;
//use tokio::process::{Command as TokioCommand};
//use tokio::io::{AsyncBufReadExt, BufReader};
use users::get_current_uid;
use tracing::{info};

use slurm_spank::{SpankHandle, spank_log_user};

use crate::{ConsoleOutput, IOData, SpankStage0, /*set_local2remote_env_var,*/ spank_getenv};

/*
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
    info!("[UID: {}] Calling: {}", &cur_uid, &fstr);

    let cmdname;
    if cur_uid == 0 {
        return Err("Cannot run stage1 as root".into());
    } else {
        if ! Path::new(&stage0.config.stage1_user_path).exists() {
            if ! Path::new(&stage0.config.stage1_system_path).exists() {
                let msg = format!("ERROR: cannot find stage1 executable at \"{}\"", &stage0.config.stage1_system_path);
                info!("{msg}");
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

    info!("Executing: {}", &cmdstr);

    let result = Command::new(cmdname)
        .args(cmdargs.clone())
        .output();

    if result.is_err() {
        return Err(format!("ERROR: Failed to spawn command: {} {:#?}", cmdname, cmdargs).into());
    };

    let output = result.unwrap();
    info!("Executed : {}", &cmdstr);

    let exit_code = match output.status.code() {
        Some(rc) => rc.to_string(),
        None => String::from("killed by signal"),
    };
    info!("RC: {}", exit_code);

    let mut stdout = String::from_utf8(output.stdout.clone()).unwrap_or(String::from(""));
    if ! stdout.is_empty() {
        stdout.pop();
        let lines = stdout.split('\n');
        for line in lines {
            info!("stdout: {}", line);
        }
    };
    let mut stderr = String::from_utf8(output.stderr.clone()).unwrap_or(String::from(""));
    if ! stderr.is_empty() {
        stderr.pop();
        let lines = stderr.split('\n');
        for line in lines {
            info!("stderr: {}", line);
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
    info!("[UID: {}] Calling: {}", &cur_uid, &fstr);

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
                info!("{msg}");
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
                info!("{msg}");
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

    info!("Executing: {}", &cmdstr);

    let result = Command::new(cmdname)
        .args(cmdargs)
        .output();

    if result.is_err() {
        return Err("failed to execute process".into());
    }
    
    let output = result.unwrap();
    info!("Executed : {}", &cmdstr);

    let exit_code = match output.status.code() {
        Some(rc) => rc.to_string(),
        None => String::from("killed by signal"),
    };
    info!("RC: {}", exit_code);

    let mut stdout = String::from_utf8(output.stdout.clone()).unwrap_or(String::from(""));
    if ! stdout.is_empty() {
        stdout.pop();
        let lines = stdout.split('\n');
        for line in lines {
            info!("stdout: {}", line);
        }
    };
    let mut stderr = String::from_utf8(output.stderr.clone()).unwrap_or(String::from(""));
    if ! stderr.is_empty() {
        stderr.pop();
        let lines = stderr.split('\n');
        for line in lines {
            info!("stderr: {}", line);
        }
    };
    
    let _ = set_current_dir(prev_dir);

    return Ok(output);
}

pub(crate) fn remote_run_stage1_new(
    stage0: &mut SpankStage0,
    spank: &mut SpankHandle,
    context: String,
    function: String,
    payload: Option<String>) 
-> Result<String, Box<dyn Error>> {

    let mut fstr = format!("{}({},{},{})", "run_stage1", context, function, "None");
    let mut pl;
    let cur_uid = get_current_uid();
    if payload.is_some() {
        pl = payload.clone().unwrap();
        fstr = format!("{}({},{},{})", "run_stage1", context, function, pl.as_str())
    }
    info!("[UID: {}] Calling: {}", &cur_uid, &fstr);

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
                info!("{msg}");
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
                info!("{msg}");
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

    info!("Executing: {}", &cmdstr);

    let mut child = match Command::new(cmdname)
        .args(cmdargs)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
        Ok(c) => c,
        Err(_) => {
            return Err("failed to spawn command".into());
        },
    };

    let stdout = match child.stdout.take() {
        Some(out) => out,
        None => {
            return Err("failed to capture stdout".into());
        },
    };
    
    let mut stderr = match child.stderr.take() {
        Some(err) => err,
        None => {
            return Err("failed to capture stderr".into());
        },
    };

    let reader = BufReader::new(stdout);

    let mut msg_out = String::from("{}");
    for ret_msg in reader.lines() {
        if ret_msg.is_ok() {
            msg_out = ret_msg.unwrap().clone();
            handle_stage1_stdout_msg(stage0, spank, &msg_out);
        }
    }

    let result = child.wait();
    info!("WAIT ENDED");

    if result.is_err() {
        return Err("failed to execute process".into());
    }
    
    let status = result.unwrap();
    info!("Executed : {}", &cmdstr);

    let exit_code = match status.code() {
        Some(rc) => rc.to_string(),
        None => String::from("killed by signal"),
    };
    info!("RC: {}", exit_code);

    let mut stderr_vec = vec![];
    let _ = stderr.read_to_end(&mut stderr_vec);
    let mut stderr = String::from_utf8(stderr_vec).unwrap_or(String::from(""));
    if ! stderr.is_empty() {
        stderr.pop();
        let lines = stderr.split('\n');
        for line in lines {
            info!("stderr: {}", line);
        }
    };

    let _ = set_current_dir(prev_dir);

    return Ok(msg_out);
}
*/

fn set_job_home_env_var(spank: &mut SpankHandle) {
    let home_dir = spank_getenv(spank, "HOME");
    if home_dir != "" {
        unsafe {
            set_var("HOME", OsStr::new(&home_dir));
        }
    }
}

fn handle_stage1_stdout_msg(
    _stage0: &mut SpankStage0,
    _spank: &mut SpankHandle,
    msg: &str)
{
    let console_out: Option<ConsoleOutput> = match serde_json::from_str(msg) {
        Ok(out) => Some(out),
        Err(_) => None,
    };

    if console_out.is_some() {
        spank_log_user!("{}", console_out.unwrap().console_out);
    };

    info!("stdout: {}", &msg);
}

pub(crate) fn run_stage1_new2(
    stage0: &mut SpankStage0,
    spank: &mut SpankHandle,
    data: &mut IOData)
-> Result<String, Box<dyn Error>> {

    let context = data.exchange.slurm_context.clone();
    let function = data.exchange.slurm_function.clone();
    let payload = data.exchange.payload.clone();

    let mut fstr = format!("{}({},{},{})", "run_stage1", context, function, "None");
    let mut pl;
    let cur_uid = get_current_uid();
    if payload.is_some() {
        pl = payload.clone().unwrap();
        fstr = format!("{}({},{},{})", "run_stage1", context, function, pl.as_str())
    }
    info!("[UID: {}] Calling: {}", &cur_uid, &fstr);

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
        if context != "remote" {
            return Err("Cannot run stage1 as root".into());
        }

        if stage0.state.username.is_none() {
            return Err("Unknown username".into());
        }
        
        if ! Path::new(&stage0.config.stage1_user_path).exists() {
            if ! Path::new(&stage0.config.stage1_system_path).exists() {
                let msg = format!("ERROR: cannot find stage1 executable at \"{}\"", &stage0.config.stage1_system_path);
                info!("{msg}");
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
                info!("{msg}");
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

    info!("Executing: {}", &cmdstr);

    let input_json = serde_json::to_string(&data)?; 
    info!("{}", &input_json);
    let (reader, mut writer) = std::io::pipe()?;

    let mut child = match Command::new(cmdname)
        .args(cmdargs)
        .stdin(reader)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
        Ok(c) => c,
        Err(_) => {
            return Err("failed to spawn command".into());
        },
    };
    //let msg = input_json.as_bytes().to_vec();
    let msg = format!("{input_json}\n").as_bytes().to_vec();
    writer.write_all(&msg)?;
    writer.flush()?;

    let stdout = match child.stdout.take() {
        Some(out) => out,
        None => {
            return Err("failed to capture stdout".into());
        },
    };
    
    let mut stderr = match child.stderr.take() {
        Some(err) => err,
        None => {
            return Err("failed to capture stderr".into());
        },
    };

    let reader = BufReader::new(stdout);

    let mut msg_out = String::from("{}");
    for ret_msg in reader.lines() {
        if ret_msg.is_ok() {
            msg_out = ret_msg.unwrap().clone();
            handle_stage1_stdout_msg(stage0, spank, &msg_out);
        }
    }

    let result = child.wait();
    info!("WAIT ENDED");

    if result.is_err() {
        return Err("failed to execute process".into());
    }
    
    let status = result.unwrap();
    info!("Executed : {}", &cmdstr);

    let exit_code = match status.code() {
        Some(rc) => rc.to_string(),
        None => String::from("killed by signal"),
    };
    info!("RC: {}", exit_code);

    let mut stderr_vec = vec![];
    let _ = stderr.read_to_end(&mut stderr_vec);
    let mut stderr = String::from_utf8(stderr_vec).unwrap_or(String::from(""));
    if ! stderr.is_empty() {
        stderr.pop();
        let lines = stderr.split('\n');
        for line in lines {
            info!("stderr: {}", line);
        }
    };

    let _ = set_current_dir(prev_dir);

    /*
    if ( context == "local" || context == "allocator" ) &&
        function == "init_post_opt" {
            set_local2remote_env_var(stage0);
    }
    */

    return Ok(msg_out);
}

