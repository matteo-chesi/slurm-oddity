use std::error::Error;
use std::env::{VarError, remove_var, var};
use nix::unistd::{getegid, geteuid};
use users::{get_current_uid, get_current_gid};

use crate::{NAME,
    SLURM_BATCH_SCRIPT,
    log::log,
    Job,
    Run,
    State,
    modify_edf_for_sbatch,
    podman_get_pid_from_file,
    remote_load_edf,
    render_user_job_config,
    send_output,
    setup_folders,
    sync_podman_pull,
    sync_podman_start,
};

pub(crate) fn slurmstepd_init_post_opt(state: &mut State) {
    remote_load_edf(state);
    let _ = job_get_info(state);
    log(&format!("JOB_INFO:\n{:#?}", state.job));
    let _ = remote_unset_env_vars(state);
}

pub(crate) fn slurmstepd_task_init(state: &mut State) {
    remote_load_edf(state);
    let _ = job_get_info(state);
    let _ = run_get_info(state);
    match render_user_job_config(state) {
        Ok(_) => {},
        Err(_) => return,
    };
    let _ = setup_folders(state);
    let _ = modify_edf_for_sbatch(state);
    log(&format!("CONFIG:\n{:#?}", state.config));
    log(&format!("RUN_INFO:\n{:#?}", state.run));
    log(&format!("JOB_INFO:\n{:#?}", state.job));
    log(&format!("JOB_ARG:\n{:#?}", state.job_arg));
    log(&format!("JOB_ENV:\n{:#?}", state.job_env));
    log(&format!("EDF_INFO:\n{:#?}", state.edf));
    log("YUPPIE!");
    let _ = sync_podman_pull(state);
    log("YEAH!!");
    let _ = sync_podman_start(state);
    log("STAKAZZO");
    send_output(state);
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
                    log("failed to unset variable {key}");
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

    let pid = match podman_get_pid_from_file(state) {
        Ok(s) => s,
        Err(_) => usize::MAX,
    };

    let run = Run {
        name: name,
        pid: pid,
        podman_tmp_path: podman_tmp_path,
        syncfile_path: syncfile_path,
    };

    state.run = Some(run);

    Ok(())
}
