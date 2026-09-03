use std::io::stdin;
use crate::{State, local_load_edf};
use raster::{update_config_by_user};
use tracing::info;

pub(crate) fn srun_init_post_opt(state: &mut State) {
    local_load_edf(state);
    let _ = update_config_by_user(&mut state.config, state.edf.clone().unwrap());
}

pub(crate) fn srun_init(state: &mut State) {
    info!("NOOOO!!");
    let lines = stdin().lines();
    for line in lines {
      info!("{}", line.unwrap());
    }
    info!("YESSS!!");
}
