use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Stage0State {
    pub(crate) enabled: bool,
}

impl Default for Stage0State {
    fn default() -> Self {
        Stage0State {
            enabled: true,
        }
    }
}
