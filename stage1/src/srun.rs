use raster::{update_config_by_user};
use tracing::{error, info};

use crate::{
    State,
    get_iodata_from_stdin,
    local_load_edf,
    send_iodata_to_stdout,
};

pub(crate) fn srun_init_post_opt(state: &mut State) {
    // Read Input
    let mut data = match get_iodata_from_stdin() {
        Ok(d) => d,
        Err(e) => {
            error!("Error: cannot read input data: {e}");
            return;
        },
    };

    info!("INPUT:\n{:#?}", data);
    
    local_load_edf(state);
    let _ = update_config_by_user(&mut state.config, state.edf.clone().unwrap());
    
    data.exchange.stage1_next_function = String::from("end");

    info!("OUTPUT:\n{:#?}", data);

    match send_iodata_to_stdout(&data) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: cannot send output data: {e}");
            return;
        },
    };
}

pub(crate) fn srun_init(_state: &mut State) {
    // Read Input
    let mut data = match get_iodata_from_stdin() {
        Ok(d) => d,
        Err(e) => {
            error!("Error: cannot read input data: {e}");
            return;
        },
    };

    info!("INPUT:\n{:#?}", data);

    data.exchange.stage1_next_function = String::from("init_post_opt");

    info!("OUTPUT:\n{:#?}", data);

    match send_iodata_to_stdout(&data) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: cannot send output data: {e}");
            return;
        },
    };
}
