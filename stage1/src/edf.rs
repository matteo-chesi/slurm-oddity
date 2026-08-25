use std::env::var;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use raster::{EDF, render, mount::SarusMount};
use crate::{SLURM_BATCH_SCRIPT, LOCAL2REMOTE_VARNAME, State, log::log, get_cache_dir_path};

pub(crate) fn local_load_edf(state: &mut State) {
    let edf_name = match &state.args.payload {
        Some(name) => String::from(name),
        None => {
            log(&format!("Error: cannot read payload argument"));
            return ();
        }
    };

    let edf = match render(edf_name.clone()) {
        Ok(f) => f,
        Err(_) => {
            log(&format!("Error: cannot render edf \"{edf_name}\""));
            return ();
        },
    };

    //edf2cache(&edf);
    add_edf_to_local2remote_data_file(&edf);
    state.edf = Some(edf);
}

pub(crate) fn remote_load_edf(state: &mut State) {
    let edf = match get_edf_from_local2remote_data_env() {
        Ok(v) => v,
        Err(_) => {
            log(&format!("Error: cannot parse EDF from environment variable"));
            return ();
        },
    };

    /*
    let edf = match cache2edf() {
        Ok(v) => v,
        Err(_) => {
            log(&format!("Error: cannot parse EDF from cache"));
            return ();
        },

    };
    */
    
    state.edf = Some(edf);
}


fn add_edf_to_local2remote_data_file(edf: &EDF) {
    
    let cache_dir_path = get_cache_dir_path();
    let l2r_data_file_path = format!("{cache_dir_path}/local2remote_data.json");

    let edf_content = match serde_json::to_string(&edf) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot serialize EDF to json");
        }
    };

    let mut file = match File::create(&l2r_data_file_path) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot open {l2r_data_file_path}");
        }
    };

    let _ = match file.write_all(edf_content.as_bytes()) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot write to {l2r_data_file_path}");
        }
    };
    let _ = file.flush();
    let _ = file.sync_all();
}

fn get_edf_from_local2remote_data_env() -> Result<EDF, String> {
    let l2r_str = match var(LOCAL2REMOTE_VARNAME) {
        Ok(s) => s,
        Err(_) => {
            log(&format!("cannot read {LOCAL2REMOTE_VARNAME} environment variable"));
            return Err(format!("cannot read {LOCAL2REMOTE_VARNAME} environment variable"));
            },
    };
    
    let edf: EDF = match serde_json::from_str(&l2r_str) {
        Ok(f) => f,
        Err(e) => {
            log(&format!("couldn't parse {LOCAL2REMOTE_VARNAME} environment variable value as a valid EDF: {e}"));
            return Err(format!("couldn't parse {LOCAL2REMOTE_VARNAME} environment variable value as a valid EDF: {e}"));
        },
    };
    Ok(edf)
}
/*
fn edf2cache(edf: &EDF) {

    let cache_dir_path = get_cache_dir_path();
    let edf_cache_file_path = format!("{cache_dir_path}/edf.json");

    let edf_content = match serde_json::to_string(&edf) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot serialize EDF to json");
        }
    };

    let mut file = match File::create(&edf_cache_file_path) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot open {edf_cache_file_path}");
        }
    };

    let _ = match file.write_all(edf_content.as_bytes()) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot write to {edf_cache_file_path}");
        }
    };
}

fn cache2edf() -> Result<EDF, String> {
    let cache_dir_path = get_cache_dir_path();
    let edf_cache_file_path = format!("{cache_dir_path}/edf.json");

    let file_path = Path::new(&edf_cache_file_path);

    if ! file_path.exists() {
        return Err(format!("cannot find file {edf_cache_file_path}"));
    };

    let content = match read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("cannot read file {edf_cache_file_path}: {e}"));
        },
    };

    let edf: EDF = match serde_json::from_str(&content) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("couldn't parse {edf_cache_file_path} value as a valid EDF: {e}"));
        },
    };
    Ok(edf)
}
*/

pub(crate) fn modify_edf_for_sbatch(
    state: &mut State,
) -> Result<(), Box<dyn Error>> {
    let job = match &state.job {
        Some(j) => j,
        None => {
            let msg = "Error: cannot find job data at this stage";
            log(msg);
            return Ok(());
        }
    };

    let stepid = job.stepid;
    if stepid == SLURM_BATCH_SCRIPT {
        let mut edf = match state.edf.clone() {
            Some(e) => e,
            None => {
                return Ok(());
            }
        };

        let argv = &state.job_arg;

        let sbatch_script = match argv.get(0) {
            Some(s) => s,
            None => {
                let msg = "Error: cannot read job argv[0]";
                log(msg);
                return Ok(());
            }
        };

        let flags = String::from("bind,ro,nosuid,nodev,private");
        let mount_string = format!("{}:{}:{}", &sbatch_script, &sbatch_script, &flags);

        let sm = match SarusMount::try_new(mount_string, &None) {
            Ok(ok) => ok,
            Err(_) => {
                let msg = "Error: cannot create sbatch script mount defintion";
                log(msg);
                return Ok(());
            }
        };

        log(&format!("NEW MOUNT: {}", sbatch_script));
        edf.mounts.append(&mut vec![sm]);

        state.edf = Some(edf);
    }    
    Ok(())
}
