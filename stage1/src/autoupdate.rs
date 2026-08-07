
use crate::{State, VERSION, log}

pub(crate) fn auto_update(state: &State) {
    let cur_ver = get_current_version();
    log("Current stage1 version is {}", cur_ver);
}

fn get_current_version() -> String {
    return String::from(VERSION);
}
