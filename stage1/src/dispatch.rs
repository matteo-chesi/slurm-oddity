use crate::{
    IOData,
    //Context,
    State,
    //Function
};
use crate::srun::{srun_init, srun_init_post_opt};
use crate::slurmstepd::{slurmstepd_init, slurmstepd_init_post_opt, slurmstepd_task_init, slurmstepd_task_exit, slurmstepd_exit};

/*
pub(crate) fn dispatch_execution(state: &mut State, data: &mut IOData) {
    if state.args.context == Context::Local {
      match state.args.function {
        Function::Init => srun_init(state, data),
        Function::InitPostOpt => srun_init_post_opt(state, data),
        _ => {}, 
      }
    }
    if state.args.context == Context::Remote {
      match state.args.function {
        Function::Init => slurmstepd_init(state, data),
        Function::InitPostOpt => slurmstepd_init_post_opt(state, data),
        Function::TaskInit => slurmstepd_task_init(state, data),
        _ => {}, 
      }
    }
}
*/

pub(crate) fn dispatch_execution(state: &mut State, data: &mut IOData) {
    if data.exchange.slurm_context == "local" {
      match data.exchange.slurm_function.as_str() {
        "init" => srun_init(state, data),
        "init_post_opt" => srun_init_post_opt(state, data),
        _ => {}, 
      }
    }
    if data.exchange.slurm_context == "remote" {
      match data.exchange.slurm_function.as_str() {
        "init" => slurmstepd_init(state, data),
        "init_post_opt" => slurmstepd_init_post_opt(state, data),
        "task_init" => slurmstepd_task_init(state, data),
        "task_exit" => slurmstepd_task_exit(state, data),
        "exit" => slurmstepd_exit(state, data),
        _ => {}, 
      }
    }
}

