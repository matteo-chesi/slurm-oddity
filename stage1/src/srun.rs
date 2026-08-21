use crate::{State, local_load_edf};
use raster::{update_config_by_user};

pub(crate) fn srun_init_post_opt(state: &mut State) {
    local_load_edf(state);
    let _ = update_config_by_user(&mut state.config, state.edf.clone().unwrap());
}
