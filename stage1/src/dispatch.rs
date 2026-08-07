use crate::{State, Function};
use crate::srun::srun_init;

pub(crate) fn dispatch_execution(state: &State) {
    if state.args.function == Function::Init {
        srun_init(state);
    }
}

