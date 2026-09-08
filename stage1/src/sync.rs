use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::path::{Path, PathBuf};
use sysinfo::{Pid, ProcessStatus, System};
use tracing::{info};
use crate::{
    PODMAN_PIDFILE_NAME,
    State,
    get_local_task_id,
    podman_pull,
    podman_start,
    podman_stop,
};

const PODMAN_START_FAILURE_FILE: &str = ".podman-start.failed";

#[derive(Debug, PartialEq, Eq)]
enum PodmanStartState {
    Pending,
    Failed,
    Started(usize),
}

pub(crate) fn is_local_task_0(state: &mut State) -> bool {
    let job = match state.job.clone() {
        Some(j) => j,
        None => {
            return false;
        }
    };

    if job.local_task_id == 0 {
        return true;
    }

    return false;
}

pub(crate) fn is_global_task_0(state: &mut State) -> bool {
    let job = match state.job.clone() {
        Some(j) => j,
        None => {
            return false;
        }
    };

    if job.global_task_id == 0 {
        return true;
    }

    return false;
}

pub(crate) fn is_node_0(state: &mut State) -> bool {
    let job = match state.job.clone() {
        Some(j) => j,
        None => {
            return false;
        }
    };

    if job.nodeid == 0 {
        return true;
    }

    return false;
}


pub(crate) fn sync_podman_pull(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    info!("SYNC PODMAN PULL START");
    if is_global_task_0(state) {
        match podman_pull(state) {
            Ok(_) => {
                sync_podman_pull_done(state, 0)?;
            }
            Err(e) => {
                sync_podman_pull_done(state, -1)?;
                return Err(e);
            }
        }
    } else {
        sync_podman_pull_wait(state)?;
    }

    info!("SYNC PODMAN PULL END");
    Ok(())
}

pub(crate) fn sync_podman_pull_wait(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    info!("task {} - waiting on image importer", get_local_task_id(state));

    let run = match state.run.clone() {
        Some(r) => r,
        None => {
            let msg = "Error: cannot find run structure";
            info!(msg);
            return Err(msg.into());
        }
    };

    let file_path = run.syncfile_path.clone();
    let pause = std::time::Duration::new(1, 0);
    while std::fs::metadata(&file_path).is_err() {
        std::thread::sleep(pause);
    }

    let f = File::open(file_path)?;
    let mut reader = BufReader::new(f);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let line = line.trim_end();
    let result = line.parse::<i32>().unwrap();
 
    info!("task {} - image importer exited with {}", get_local_task_id(state), result);

    if result != 0 {
        let msg = format!("Error: image importer exited with {result}");
        info!("{msg}");
        return Err(msg.into());
    }

    Ok(())
}

pub(crate) fn sync_podman_pull_done(
    state: &mut State,
    result: i32,
) -> Result<(), Box<dyn Error>> {
    let run = match state.run.clone() {
        Some(r) => r,
        None => {
            let msg = "Error: cannot find run structure";
            info!(msg);
            return Err(msg.into());
        }
    };
    info!("task {} - image importer completed with {} - communicating", get_local_task_id(state), result);

    let mut file = File::create(run.syncfile_path)?;
    write!(file, "{}\n", result)?;

    Ok(())
}

pub(crate) fn sync_podman_start(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    info!("SYNC PODMAN START START");
    if is_local_task_0(state) {
        info!("SYNC PODMAN START TASK0 START");
        let run = state
            .run
            .as_ref()
            .ok_or_else(|| -> Box<dyn Error> { "couldn't find run".into() })?;
        let tmp_path = PathBuf::from(&run.podman_tmp_path);
        clear_podman_start_failure(&tmp_path)?;

        if let Err(error) = podman_start(state) {
            info!("SYNC PODMAN START TASK0 ERROR");
            if let Err(sync_error) = notify_podman_start_failure(&tmp_path) {
                info!("failed to notify sibling tasks of Podman startup failure: {}", sync_error);
            }
            return Err(error);
        }
    }
    info!("SYNC PODMAN START WAIT START");
    sync_podman_start_wait(state)?;
    info!("SYNC PODMAN START WAIT END");
    info!("SYNC PODMAN START END");

    Ok(())
}

pub(crate) fn sync_podman_start_wait(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let run = match &state.run {
        Some(o) => o,
        None => {
            let msg = "couldn't find run";
            info!(msg);
            return Err(msg.into());
        }
    };

    let tmp_path = PathBuf::from(&run.podman_tmp_path);
    let pid;

    let mut attempts: u32 = 0;
    let pause = std::time::Duration::from_millis(100);
    // Wait max 1 minute for pidfile
    let mut max_attempts: u32 = 600;

    loop {
        match read_podman_start_state(&tmp_path)? {
            PodmanStartState::Started(started_pid) => {
                pid = started_pid;
                break;
            }
            PodmanStartState::Failed => {
                let msg = "Podman container startup failed on local task 0";
                info!(msg);
                return Err(msg.into());
            }
            PodmanStartState::Pending => {
                attempts += 1;

                // Fail with error after max attempts
                if attempts >= max_attempts {
                    let msg = String::from(
                        "Timed out waiting for Podman container to start on local task 0",
                    );
                    let msg = format!("task {} - {msg}", get_local_task_id(state));
                    info!("{msg}");
                    return Err(msg.into());
                }

                // Log first and every 50 retries to limit log spam
                if attempts == 1 || attempts.is_multiple_of(50) {
                    info!("task {} - Still waiting for Podman container to start on local task 0", get_local_task_id(state));
                }

                std::thread::sleep(pause);
            }
        }
    }

    attempts = 0;
    // Wait max 5 minutes for entrypoint
    max_attempts = 5 * 600;

    loop {
        if is_process_stopped(pid)? {
            break;
        } else {
            attempts += 1;

            // Fail with error after max attempts
            if attempts >= max_attempts {
                let msg = format!("container entrypoint process {pid} did not complete.");
                let logmsg = format!("task {} - {msg}", get_local_task_id(state));
                info!("{logmsg}");
                return Err(msg.into());
            }

            // Log first and every 50 retries to limit log spam
            if attempts == 1 || attempts.is_multiple_of(50) {
                info!(
                    "task {} - container entrypoint process {pid} hasn't completed yet, waiting and retrying",
                    get_local_task_id(state)
                );
            }

            std::thread::sleep(pause);
        }
    }

    let mut newrun = state.run.clone().unwrap();
    newrun.pid = pid;
    info!("Container PID: {pid}");

    state.run = Some(newrun);

    Ok(())
}

fn clear_podman_start_failure(run_path: &Path) -> Result<(), Box<dyn Error>> {
    let path = podman_start_failure_path(run_path);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to remove stale Podman startup failure marker `{}`: {}",
            path.display(),
            error
        )
        .into()),
    }
}

fn notify_podman_start_failure(run_path: &Path) -> Result<(), Box<dyn Error>> {
    let path = podman_start_failure_path(run_path);
    File::create(&path).map(|_| ()).map_err(|error| {
        format!(
            "failed to create Podman startup failure marker `{}`: {}",
            path.display(),
            error
        )
        .into()
    })
}

fn read_podman_start_state(run_path: &Path) -> Result<PodmanStartState, Box<dyn Error>> {
    let failure_path = podman_start_failure_path(run_path);
    match failure_path.try_exists() {
        Ok(true) => return Ok(PodmanStartState::Failed),
        Ok(false) => {}
        Err(error) => {
            return Err(format!(
                "failed to inspect Podman startup failure marker `{}`: {}",
                failure_path.display(),
                error
            )
            .into());
        }
    }

    let pidfile = podman_pidfile_path(run_path);
    let contents = match fs::read_to_string(&pidfile) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(PodmanStartState::Pending);
        }
        Err(error) => {
            return Err(format!(
                "failed to read container PID file `{}`: {}",
                pidfile.display(),
                error
            )
            .into());
        }
    };

    let value = contents.trim();
    let pid = value.parse::<usize>().map_err(|error| {
        format!(
            "invalid PID `{}` in container PID file `{}`: {}",
            value,
            pidfile.display(),
            error
        )
    })?;
    Ok(PodmanStartState::Started(pid))
}

fn podman_start_failure_path(run_path: &Path) -> PathBuf {
    run_path.join(PODMAN_START_FAILURE_FILE)
}

fn podman_pidfile_path(run_path: &Path) -> PathBuf {
    run_path.join(PODMAN_PIDFILE_NAME)
}

fn is_process_stopped(pid: usize) -> Result<bool, Box<dyn Error>> {
    let p = Pid::from(pid);

    let s = System::new_all();
    let Some(process) = s.process(p) else {
        return Err(format!("cannot find process {pid}").into());
    };
    let state = process.status();

    if state == ProcessStatus::Stop {
        return Ok(true);
    } else {
        return Ok(false);
    }
}

pub(crate) fn sync_podman_stop(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let run = match state.run.clone() {
        Some(r) => r,
        None => {
            let msg = "Error: cannot find run structure";
            info!(msg);
            return Err(msg.into());
        }
    };

    let job = match state.job.clone() {
        Some(r) => r,
        None => {
            let msg = "Error: cannot find job structure";
            info!(msg);
            return Err(msg.into());
        }
    };

    let task_id = job.local_task_id;
    let task_count = job.local_task_count;

    // create sync folder if doesn't exist
    let completed_dir_path = format!("{}/completed", run.podman_tmp_path);
    if !std::path::Path::new(&completed_dir_path).exists() {
        std::fs::create_dir_all(&completed_dir_path)?;
    }

    // touch file in sync folder
    let completed_file_path = format!("{}/task_{}.exit", completed_dir_path, task_id);
    File::create(completed_file_path)?;

    // Wait for all tasks to stop podman.
    let readdir = std::fs::read_dir(&completed_dir_path)?;
    if (readdir.count() as u32) == task_count {
        sync_cleanup_fs_local_dir_completed(state)?;
        podman_stop(state)?;
    }

    Ok(())
}

pub(crate) fn sync_cleanup_fs_local_dir_completed(
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

    let completed_dir_path = format!("{}/completed", base_path);

    if !Path::new(&completed_dir_path).exists() {
        ()
    } else {
        match std::fs::remove_dir_all(&completed_dir_path) {
            Ok(_) => (),
            Err(e) => {
                let msg = format!(
                    "couldn't cleanup \"{:#?}\", error {}",
                    &completed_dir_path, e
                );
                info!(msg);
                return Err(msg.into());
            }
        };
    }
    Ok(())
}

pub(crate) fn sync_cleanup_fs_shared(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {

    if !is_node_0(state) {
        return Ok(());
    }

    let syncfile_path = match state.run.clone() {
        Some(r) => r.syncfile_path,
        None => {
            let msg = "Error: couldn't find syncfile_path";
            info!(msg);
            return Err(msg.into());
        }
    };

    info!("delete {}", &syncfile_path);
    match std::fs::remove_file(&syncfile_path) {
        Ok(_) => (),
        Err(e) => {
            let msg = format!(
                "couldn't cleanup syncfile_path \"{:#?}\", error {}",
                &syncfile_path, e
            );
            info!(msg);
            return Err(msg.into());
        }
    }

    if let Some(output) = raster::imagestore_keepalive(&state.config)? {
        info!("{}", output);
    }

    Ok(())
}

