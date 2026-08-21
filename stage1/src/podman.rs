use std::error::Error;
use std::path::Path;
use std::fs::read_to_string;
use crate::{State};

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

