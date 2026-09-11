use std::error::Error;
use std::env::{VarError, remove_var, var};
//use std::{thread, time::Duration};
use nix::unistd::{getegid, geteuid};
use users::{get_current_uid, get_current_gid};
use tracing::{error, info};

use crate::{
    COLOR,
    IOData,
    NAME,
    SLURM_BATCH_SCRIPT,
    DataContainer,
    Job,
    Run,
    State,
    cleanup_fs_local,
    console_output,
    //get_iodata_from_stdin,
    modify_edf_for_sbatch,
    podman_get_pid_from_file,
    remote_load_edf,
    remove_empty_log_file,
    render_user_job_config,
    send_iodata_to_stdout,
    setup_folders,
    sync_podman_pull,
    sync_podman_start,
    sync_podman_stop,
    sync_cleanup_fs_shared,
};
/*
pub(crate) fn slurmstepd_init(_state: &mut State, data: &mut IOData) {

    /*
    let mut data = match get_iodata_from_stdin() {
        Ok(d) => d,
        Err(e) => {
            error!("Error: cannot read input data: {e}");
            return;
        },
    };
    */

    info!("INPUT:\n{:#?}", data);

    data.exchange.stage1_function_set = vec![
        "init".to_string(),
        "init_post_opt".to_string(),
        "task_init".to_string(),
        "task_exit".to_string(),
        "exit".to_string(),
    ];

    info!("OUTPUT:\n{:#?}", data);

    match send_iodata_to_stdout(&data) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: cannot send output data: {e}");
            return;
        },
    };
}
*/

pub(crate) fn slurmstepd_init_post_opt(state: &mut State, data: &mut IOData) {
    
    /*
    let mut data = match get_iodata_from_stdin() {
        Ok(d) => d,
        Err(e) => {
            error!("Error: cannot read input data: {e}");
            return;
        },
    };
    */

    info!("INPUT:\n{:#?}", data);
    
    data.exchange.stage1_function_set = vec![
        "init_post_opt".to_string(),
        "task_init".to_string(),
        "task_exit".to_string(),
        "exit".to_string(),
    ];

    info!("OUTPUT:\n{:#?}", data);

    remote_load_edf(state);
    let _ = job_get_info(state);
    info!("JOB_INFO:\n{:#?}", state.job);
    let _ = remote_unset_env_vars(state);
    
    match send_iodata_to_stdout(&data) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: cannot send output data: {e}");
            return;
        },
    };
}

pub(crate) fn slurmstepd_task_init(state: &mut State, data: &mut IOData) {
    /*
    info!("OK TASK INIT");
    let mut data = match get_iodata_from_stdin() {
        Ok(d) => d,
        Err(e) => {
            error!("Error: cannot read input data: {e}");
            return;
        },
    };
    */

    info!("INPUT:\n{:#?}", data);
    
    remote_load_edf(state);
    let _ = job_get_info(state);
    let _ = run_get_info(state);
    match render_user_job_config(state) {
        Ok(_) => {},
        Err(_) => return,
    };
    let _ = setup_folders(state);
    let _ = modify_edf_for_sbatch(state);
    

    info!("CONFIG:\n{:#?}", state.config);
    info!("RUN_INFO:\n{:#?}", state.run);
    info!("JOB_INFO:\n{:#?}", state.job);
    //info!("JOB_ARG:\n{:#?}", state.job_arg);
    info!("JOB_ARG2:\n{:#?}", data.exchange.job_arg);
    //info!("JOB_ENV:\n{:#?}", state.job_env);
    info!("JOB_ENV2:\n{:#?}", data.exchange.job_env);
    info!("EDF_INFO:\n{:#?}", state.edf);
    info!("YUPPIE!");
    
    announce(data);
    //thread::sleep(Duration::from_millis(5000));
    //console_output("YEAH!!!");
    
    let _ = sync_podman_pull(state);
    info!("YEAH!!");
    let _ = sync_podman_start(state);
    info!("STAKAZZO");

    let pid: u32 = match state.run.clone().unwrap().pid.try_into() {
        Ok(p) => p,
        Err(_) => {
            error!("Error: convert pid in u32");
            return;
        },
    };
    
    let dc = DataContainer {
        env: state.edf.clone().unwrap().env,
        pid: pid,
        workdir: state.edf.clone().unwrap().workdir,
    };
    data.exchange.container = Some(dc);

    info!("OUTPUT:\n{:#?}", data);

    //send_output(state);
    
    match send_iodata_to_stdout(&data) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: cannot send output data: {e}");
            return;
        },
    }
}

pub(crate) fn slurmstepd_task_exit(state: &mut State, data: &mut IOData) {
    //info!("INPUT:\n{:#?}", data);
    
    let _ = job_get_info(state);
    let _ = run_get_info(state);
    let _ = sync_podman_stop(state);

    //info!("OUTPUT:\n{:#?}", data);

    match send_iodata_to_stdout(&data) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: cannot send output data: {e}");
            return;
        },
    }
}

pub(crate) fn slurmstepd_exit(state: &mut State, data: &mut IOData) {
    //info!("INPUT:\n{:#?}", data);
    
    let _ = job_get_info(state);
    let _ = run_get_info(state);
    let _ = cleanup_fs_local(state);
    let _ = sync_cleanup_fs_shared(state);

    //info!("OUTPUT:\n{:#?}", data);

    match send_iodata_to_stdout(&data) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: cannot send output data: {e}");
            return;
        },
    }
    
    let _ = remove_empty_log_file(state);
}

pub(crate) fn remote_unset_env_vars(state: &mut State) -> Result<(), Box<dyn Error>> {
    let edf_env = state.edf.clone().unwrap().env;
    let mut unset_keys = vec![];

    for (key, value) in edf_env.iter() {
        if value == "" {
            unset_keys.push(key);

            match var(key) {
                Err(VarError::NotPresent) => continue,
                Ok(_) | Err(_) => {},
            }

            unsafe {
                remove_var(key)
            }

            match var(key) {
                Err(VarError::NotPresent) => {},
                Err(_) | _ => {
                    info!("failed to unset variable {key}");
                    return Err("failed to unset variable: {key}")?;
                },
            }
        }
    }

    Ok(())
}

pub(crate) fn job_get_info(state: &mut State) -> Result<(), Box<dyn Error>> {
     
    let cwd = match state.job_env.get("PWD") {
        Some(id) => id.parse().unwrap_or(String::from("")),
        None => String::from(""),
    };

    let job_id: u32 = match state.job_env.get("SLURM_JOB_ID") {
        Some(id) => id.parse().unwrap_or(u32::MAX),
        None => u32::MAX,
    };

    let step_id: u32 = match state.job_env.get("SLURM_STEP_ID") {
        Some(id) => id.parse().unwrap_or(u32::MAX),
        None => u32::MAX,
    };

    let local_task_id: u32 = match state.job_env.get("SLURM_LOCALID") {
        Some(id) => id.parse().unwrap_or(u32::MAX),
        None => u32::MAX,
    };

    let global_task_id: u32 = match state.job_env.get("SLURM_PROCID") {
        Some(id) => id.parse().unwrap_or(u32::MAX),
        None => u32::MAX,
    };

    let node_id: u32 = match state.job_env.get("SLURM_NODEID") {
        Some(id) => id.parse().unwrap_or(u32::MAX),
        None => u32::MAX,
    };

    let local_task_count: u32 = match state.job_env.get("SLURM_TASKS_PER_NODE") {
        Some(id) => id.parse().unwrap_or(u32::MAX),
        None => u32::MAX,
    };

    let global_task_count: u32 = match state.job_env.get("SLURM_NTASKS") {
        Some(id) => id.parse().unwrap_or(u32::MAX),
        None => u32::MAX,
    };

    let job = Job {
        uid: get_current_uid(),
        gid: get_current_gid(),
        jobid: job_id,
        stepid: step_id,
        local_task_id: local_task_id,
        global_task_id: global_task_id,
        nodeid: node_id,
        local_task_count: local_task_count,
        total_task_count: global_task_count,
        cwd: cwd,
        euid: geteuid().as_raw(),
        egid: getegid().as_raw(),
    };

    state.job = Some(job);

    Ok(())
}

pub(crate) fn run_get_info(state: &mut State) -> Result<(), Box<dyn Error>> {
    let config = state.config.clone();
    let job = state.job.clone().unwrap();

    let mut step_name = format!("{}", job.stepid);
    if job.stepid == SLURM_BATCH_SCRIPT {
        step_name = String::from("batch");
    };

    let name = format!("{}_{}.{}", NAME, job.jobid, step_name);
    let podman_tmp_path = format!("{}/{}", config.podman_tmp_path, name);
    let syncfile_path = format!("{}/.{}_import.done", config.parallax_imagestore, name);

    let mut run = Run {
        name: name,
        pid: usize::MAX,
        podman_tmp_path: podman_tmp_path,
        syncfile_path: syncfile_path,
    };

    state.run = Some(run.clone());

    run.pid = match podman_get_pid_from_file(state) {
        Ok(s) => s,
        Err(_) => usize::MAX,
    };
    
    state.run = Some(run.clone());

    Ok(())
}

pub(crate) fn announce(data: &mut IOData) {

    let status;

    if ! data.forward.as_ref().unwrap().config.poison {
        status = "living";
    } else {
        status = "dead";
    }

    console_output(&format!("I am {} {} cat!\n", COLOR, status));
}
