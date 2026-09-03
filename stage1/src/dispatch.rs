use crate::{Context, State, Function};
use crate::srun::{srun_init, srun_init_post_opt};
use crate::slurmstepd::{slurmstepd_init_post_opt, slurmstepd_task_init};

pub(crate) fn dispatch_execution(state: &mut State) {
    if state.args.context == Context::Local {
      match state.args.function {
        Function::Init => srun_init(state),
        Function::InitPostOpt => srun_init_post_opt(state),
        _ => {}, 
      }
    }
    if state.args.context == Context::Remote {
      match state.args.function {
        Function::InitPostOpt => slurmstepd_init_post_opt(state),
        Function::TaskInit => slurmstepd_task_init(state),
        _ => {}, 
      }
    }
}

