use raster::{update_config_by_user};
use tracing::{error, info};

use crate::{
    IOData,
    State,
    //get_iodata_from_stdin,
    local_load_edf,
    send_iodata_to_stdout,
};

pub(crate) fn srun_init_post_opt(state: &mut State, data: &mut IOData) {
    /*
    // Read Input
    let mut data = match get_iodata_from_stdin() {
        Ok(d) => d,
        Err(e) => {
            error!("Error: cannot read input data: {e}");
            return;
        },
    };
    */

    info!("INPUT:\n{:#?}", data);
    
    local_load_edf(state, data);
    let _ = update_config_by_user(&mut state.config, state.edf.clone().unwrap());
    
    info!("OUTPUT:\n{:#?}", data);

    match send_iodata_to_stdout(&data) {
        Ok(_) => {},
        Err(e) => {
            error!("Error: cannot send output data: {e}");
            return;
        },
    };
}

pub(crate) fn srun_init(_state: &mut State, data: &mut IOData) {
    /*
    // Read Input
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
