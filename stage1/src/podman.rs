use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs::read_to_string;
use std::time::Instant;
use tracing::{info};

use sarus_suite_podman_driver::{self as pmd, ContainerCtx, PodmanCtx};

use crate::{State, setup_imagestore};

pub(crate) const PODMAN_PIDFILE_NAME: &str = "pidfile";

pub(crate) fn podman_get_pid_from_file(state: &mut State) -> Result<usize, Box<dyn Error>> {
    let run = match &state.run {
        Some(o) => o,
        None => {
            return Err("couldn't find run data".into());
        }
    };

    //Try to read from pidfile
    let pidfile = format!("{}/pidfile", run.podman_tmp_path);
    if Path::new(&pidfile).exists() {
        let strpid = match read_to_string(&pidfile) {
            Ok(s) => s,
            Err(_) => {
                let err_msg = format!("cannot read pid from {pidfile}");
                return Err(err_msg.into());
            }
        };
        let pid: usize = match strpid.parse() {
            Ok(p) => p,
            Err(_) => {
                let err_msg = format!("cannot convert {strpid} to number");
                return Err(err_msg.into());
            }
        };
        return Ok(pid);
    } else {
        let err_msg = format!("{pidfile} NOT FOUND!");
        Err(err_msg.into())
    }
}

pub(crate) fn podman_pull(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let edf = match &state.edf {
        Some(o) => o,
        None => {
            let err_msg = format!("couldn't find edf");
            return Err(err_msg.into());
        }
    };

    let run = match &state.run {
        Some(o) => o,
        None => {
            let err_msg = format!("couldn't find run");
            return Err(err_msg.into());
        }
    };

    let config = &state.config;
    setup_imagestore(config)?;

    let graphroot = format!("{}/graphroot", run.podman_tmp_path);
    let runroot = format!("{}/runroot", run.podman_tmp_path);

    let ro_ctx = PodmanCtx {
        podman_path: PathBuf::from(&config.podman_path),
        module: None,
        graphroot: Some(PathBuf::from(&graphroot)),
        runroot: Some(PathBuf::from(&runroot)),
        parallax_mount_program: None,
        ro_store: Some(PathBuf::from(&config.parallax_imagestore)),
        podman_env: None,
    }
    .with_env("PARALLAX_MP_UID", config.parallax_mp_uid.to_string())
    .with_env("PARALLAX_MP_GID", config.parallax_mp_gid.to_string())
    .with_env(
        "PARALLAX_MP_SQUASHFUSE_CMD",
        config.parallax_mp_squashfuse_path.clone(),
    )
    .with_env("PARALLAX_MP_LOGFILE", config.parallax_mp_logfile.clone());

    let local_ctx = PodmanCtx {
        podman_path: PathBuf::from(&config.podman_path),
        module: None,
        graphroot: Some(PathBuf::from(&graphroot)),
        runroot: Some(PathBuf::from(&runroot)),
        parallax_mount_program: None,
        ro_store: None,
        podman_env: None,
    }
    .with_env("PARALLAX_MP_UID", config.parallax_mp_uid.to_string())
    .with_env("PARALLAX_MP_GID", config.parallax_mp_gid.to_string())
    .with_env(
        "PARALLAX_MP_SQUASHFUSE_CMD",
        config.parallax_mp_squashfuse_path.clone(),
    )
    .with_env("PARALLAX_MP_LOGFILE", config.parallax_mp_logfile.clone());

    let migrate_ctx = PodmanCtx {
        podman_path: PathBuf::from(&config.podman_path),
        module: None,
        graphroot: Some(PathBuf::from(&graphroot)),
        runroot: None,
        parallax_mount_program: None,
        ro_store: Some(PathBuf::from(&config.parallax_imagestore)),
        podman_env: None,
    }
    .with_env("PARALLAX_MP_UID", config.parallax_mp_uid.to_string())
    .with_env("PARALLAX_MP_GID", config.parallax_mp_gid.to_string())
    .with_env(
        "PARALLAX_MP_SQUASHFUSE_CMD",
        config.parallax_mp_squashfuse_path.clone(),
    )
    .with_env("PARALLAX_MP_LOGFILE", config.parallax_mp_logfile.clone());

    if !pmd_image_exists(&edf.image, &ro_ctx)? {
        info!(
            "pulling image \"{}\" from remote in local graphroot",
            edf.image
        );
        pmd_pull(&edf.image, &local_ctx)?;

        if !pmd_image_exists(&edf.image, &local_ctx)? {
            let msg = "Error: podman pull failed, cannot find image in local graphroot";
            info!("{msg}");
            return Err(msg.into());
        }

        info!("migrating image \"{}\" to shared imagestore", edf.image);
        pmd_parallax_migrate(&config.parallax_path, &migrate_ctx, &edf.image)?;

        info!("removing image \"{}\" from local graphroot", edf.image);
        if let Err(error) = pmd_rmi(&edf.image, &local_ctx) {
            info!(
                "Error: failed to remove image \"{}\" from local graphroot: {}",
                edf.image,
                error
            );
        }

        if !pmd_image_exists(&edf.image, &ro_ctx)? {
            let msg = "Error: couldn't find image on shared imagestore after migration";
            info!("{msg}");
            return Err(msg.into());
        }
    }

    Ok(())
}

pub(crate) fn pmd_image_exists(image: &str, ctx: &PodmanCtx) -> pmd::Result<bool> {
    pmd::image_exists(image, Some(ctx))
}

pub(crate) fn pmd_pull(image: &str, ctx: &PodmanCtx) -> pmd::Result<()> {
    pmd::pull(image, Some(ctx))
}

pub(crate) fn pmd_parallax_migrate(
    parallax_path: &str,
    ctx: &PodmanCtx,
    image: &str,
) -> Result<(), Box<dyn Error>> {
    pmd::parallax_migrate(&PathBuf::from(parallax_path), ctx, image)?;
    Ok(())
}

pub(crate) fn pmd_rmi(image: &str, ctx: &PodmanCtx) -> pmd::Result<()> {
    pmd::rmi(image, Some(ctx))
}

pub(crate) fn podman_start(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let edf = match &state.edf {
        Some(o) => o,
        None => {
            return Err("couldn't find edf".into());
        }
    };

    let run = match &state.run {
        Some(o) => o,
        None => {
            return Err("couldn't find run".into());
        }
    };

    let job = match &state.job {
        Some(job) => job,
        None => {
            return Err("couldn't find job".into());
        }
    };

    let config = state.config.clone();

    let graphroot = format!("{}/graphroot", run.podman_tmp_path);
    let runroot = format!("{}/runroot", run.podman_tmp_path);
    let pidfile = format!("{}/{}", run.podman_tmp_path, PODMAN_PIDFILE_NAME);
    let command = vec!["sh", "-c", "kill -STOP $$ ; exit 0"];

    let c_ctx = ContainerCtx {
        name: run.name.clone(),
        interactive: false,
        tty: false,
        detach: true,
        auto_remove: true,
        set_env: false,
        pidfile: Some(PathBuf::from(pidfile.clone())),
        user: Some(job.uid.to_string()),
    };

    let run_ctx = PodmanCtx {
        podman_path: PathBuf::from(&config.podman_path),
        module: Some(String::from(&config.podman_module)),
        graphroot: Some(PathBuf::from(&graphroot)),
        runroot: Some(PathBuf::from(&runroot)),
        parallax_mount_program: Some(PathBuf::from(&config.parallax_mount_program)),
        ro_store: Some(PathBuf::from(&config.parallax_imagestore)),
        podman_env: None,
    }
    .with_env("PARALLAX_MP_UID", config.parallax_mp_uid.to_string())
    .with_env("PARALLAX_MP_GID", config.parallax_mp_gid.to_string())
    .with_env(
        "PARALLAX_MP_SQUASHFUSE_CMD",
        config.parallax_mp_squashfuse_path.clone(),
    )
    .with_env("PARALLAX_MP_LOGFILE", config.parallax_mp_logfile.clone());

    info!(
        "mount env: PARALLAX_MP_UID={} PARALLAX_MP_GID={}",
        config.parallax_mp_uid.to_string(),
        config.parallax_mp_gid.to_string()
    );

    return pmd_run(&edf, &config, &run_ctx, &c_ctx, command);
}

pub(crate) fn pmd_run<I, S>(
    edf: &raster::EDF,
    config: &raster::Config,
    p_ctx: &PodmanCtx,
    c_ctx: &ContainerCtx,
    cmd: I,
) -> Result<(), Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let t0 = Instant::now();
    info!("PODMAN RUN START");
    let result = pmd::run_from_edf_output(edf, Some(p_ctx), c_ctx, cmd);
    let tend = t0.elapsed();

    if config.perfmon {
        info!(
            "skybox-perf: Podman run elapsed time: {:.6} sec",
            tend.as_secs_f64()
        );
    }

    info!("PODMAN RUN RESULT {:#?}", result);
    result?;
    info!("PODMAN RUN END");
    Ok(())
}
