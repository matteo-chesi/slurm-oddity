use std::error::Error;

use slurm_spank::{Context, Plugin, SpankHandle};

use crate::{SpankStage0, run_stage1};
use crate::args::*;
use crate::config::{dispatch_load_config};
use crate::state::{dispatch_load_state};

unsafe impl Plugin for SpankStage0 {
    fn init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        //if ! Path::new(&self.config.stage1).exists() {
        //    log(format!("ERROR: cannot find {}", &self.config.stage1).as_str());
        //}

        let context;
        let function = String::from("init");
        let payload = None;

        match spank.context()? {
            
            Context::Slurmd => {
                //let _ = slurmd_init(self, spank)?;
                context  = String::from("slurmd");
            }
            
            Context::Local => {
                //let _ = srun_init(self, spank)?;
                context  = String::from("local");
            }
            
            Context::Allocator => {
                //let _ = alloc_init(self, spank)?;
                context  = String::from("allocator");
            }
            Context::Remote => {
                //let _ = slurmstepd_init(self, spank)?;
                context  = String::from("remote");
            }
            _ => { return Ok(()); }
        }

        dispatch_load_config(self, spank)?;
        dispatch_load_state(self, spank)?;

        let _ = register_plugin_args(spank)?;

        run_stage1(self, spank, context, function, payload);

        Ok(())
    }
    
    fn init_post_opt(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}

        let context;
        let function = String::from("init_post_opt");

        match spank.context()? {
            Context::Local => {
                //let _ = srun_init_post_opt(self, spank)?;
                context  = String::from("local");
            }
            Context::Allocator => {
                //let _ = alloc_init_post_opt(self, spank)?;
                context  = String::from("allocator");
            }
            Context::Remote => {
                //let _ = slurmstepd_init_post_opt(self, spank)?;
                context  = String::from("remote");
            }
            _ => { return Ok(()); }
        }

        load_plugin_args(self, spank)?;
        let payload = self.args.payload.clone();
        run_stage1(self, spank, context, function, payload);

        Ok(())
    }

    fn user_init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}

        let context  = String::from("remote");
        let function = String::from("user_init");
        let payload = self.args.payload.clone();

        //slurmstepd_user_init(self, spank)
        run_stage1(self, spank, context, function, payload);
        Ok(())
    }

    fn task_init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}
        
        let context  = String::from("remote");
        let function = String::from("task_init");
        let payload = self.args.payload.clone();

        //slurmstepd_task_init(self, spank)
        //task_init_adjust(self, spank)?;
        run_stage1(self, spank, context, function, payload);
        Ok(())
    }

    fn exit(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}
        let context;
        let function = String::from("exit");
        let payload = self.args.payload.clone();

        match spank.context()? {
            Context::Slurmd => {
                //let _ = slurmd_exit(self, spank)?;
                context  = String::from("slurmd");
            }
            Context::Local => {
                //let _ = srun_exit(self, spank)?;
                context  = String::from("local");
            }
            Context::Allocator => {
                //let _ = alloc_exit(self, spank)?;
                context  = String::from("allocator");
            }
            Context::Remote => {
                //let _ = slurmstepd_exit(self, spank)?;
                context  = String::from("remote");
            }
            _ => { return Ok(()); }
        }

        run_stage1(self, spank, context, function, payload);
        Ok(())
    }

    /*
    fn slurmd_exit(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}

        let context  = String::from("slurmd");
        let function = String::from("exit");
        let payload = None;

        //slurmd_exit(self, spank)
        run_stage1(self, spank, context, function, payload);
        Ok(())
    }
    */

    fn task_exit(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}
        
        let context  = String::from("remote");
        let function = String::from("task_exit");
        let payload = self.args.payload.clone();

        //slurmstepd_task_exit(self, spank)
        run_stage1(self, spank, context, function, payload);
        Ok(())
    }

    fn task_init_privileged(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}
        
        let context;
        let function = String::from("task_init_privileged");
        let payload = self.args.payload.clone();

        match spank.context()? {
            Context::Remote => {
                //let _ = slurmstepd_task_init_privileged(self, spank)?;
                context  = String::from("remote");
            }
            _ => { return Ok(()); }
        }

        run_stage1(self, spank, context, function, payload);
        Ok(())
    }
    
}
