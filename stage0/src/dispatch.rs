use std::error::Error;
use std::fs::OpenOptions;
use std::sync::Mutex;
use slurm_spank::{Context, Plugin, SpankHandle, spank_log_user, spank_log_error};
use tracing_subscriber::fmt;
use tracing::{Level, info};

use crate::{
    APP_NAME,
    //IOData,
    ErrorDestination,
    SpankStage0,
    error_destination,
    format_error_chain,
    //get_iodata,
    get_iodata_from_str,
    init_log_file,
    //run_stage1,
    run_stage1_new2,
    //remote_run_stage1_new,
    remove_empty_log_file,
    set_panic_hook,
    set_local2remote_env_var_from_iodata,
    update_iodata,
};
use crate::args::*;
use crate::config::{dispatch_load_config};
use crate::state::{dispatch_load_state};
use crate::containers::{
    //container_join_from_stage1_output,
    container_join_from_iodata,
};

macro_rules! log_init {
    () => {
        let span = tracing::span!(tracing::Level::INFO, APP_NAME);
        let _ = span.enter();
    };
}

unsafe impl Plugin for SpankStage0 {
    fn init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        log_init!();
        /*
        info!("PROVA!");

        let context;
        let function = String::from("init");
        let payload = None;

        match spank.context()? {
            Context::Local => {
                tracing::error!("STIKAZZZZZZI!!!");
                context  = String::from("local");
            }
            
            Context::Allocator => {
                context  = String::from("allocator");
            }
            Context::Remote => {
                context  = String::from("remote");
            }
            _ => { return Ok(()); }
        }

        // Required to understand where to log.
        dispatch_load_state(self, spank)?;
        dispatch_load_config(self, spank)?;
        let mut io_data = get_iodata(self, spank, context.clone(), function.clone(), payload.clone())?;
        */


        let _ = register_plugin_args(spank)?;
        /*
        let output = run_stage1_new2(self, spank, &mut io_data)?;
        let json_value: serde_json::Value = serde_json::from_str(&output)?;
        info!("OUTPUT:\n{:#?}", serde_json::to_string_pretty(&json_value));
        self.iodata = get_iodata_from_str(&output)?;
        set_local2remote_env_var_from_iodata(&mut self.iodata);
        */

        Ok(())
    }
    
    fn init_post_opt(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        log_init!();
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
        if payload.is_none() {
            self.state.enabled = false;
            return Ok(());
        }
        // Required to understand where to log.
        dispatch_load_state(self, spank)?;
        dispatch_load_config(self, spank)?;
        //let mut io_data = get_iodata(self, spank, context.clone(), function.clone(), payload.clone())?;

        update_iodata(self, spank, context.clone(), function.clone(), payload.clone())?;
        info!("IODATA:\n{}", serde_json::to_string_pretty(&self.iodata)?);
        
        /*
        if ! self.iodata.clone().unwrap().exchange.stage1_function_set.contains(&function) {
            return Ok(());
        }
        */

        //let _ = run_stage1(self, spank, context, function, payload);
        let mut io_data = self.iodata.clone().unwrap();
        let output = run_stage1_new2(self, spank, &mut io_data)?;
        self.iodata = get_iodata_from_str(&output)?;
        set_local2remote_env_var_from_iodata(&mut self.iodata);

        Ok(())
    }

    fn user_init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        log_init!();
        if ! self.state.enabled {
            return Ok(());
        }
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}

        let context  = String::from("remote");
        let function = String::from("user_init");
        let payload = self.args.payload.clone();
        update_iodata(self, spank, context.clone(), function.clone(), payload.clone())?;

        if ! self.iodata.clone().unwrap().exchange.stage1_function_set.contains(&function) {
            return Ok(());
        }

        //slurmstepd_user_init(self, spank)
        //let _ = run_stage1(self, spank, context, function, payload);
        let mut io_data = self.iodata.clone().unwrap();
        let output = run_stage1_new2(self, spank, &mut io_data)?;
        self.iodata = get_iodata_from_str(&output)?;
        
        Ok(())
    }

    fn task_init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        log_init!();
        if ! self.state.enabled {
            return Ok(());
        }
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}
        
        let context  = String::from("remote");
        let function = String::from("task_init");
        let payload = self.args.payload.clone();

        update_iodata(self, spank, context.clone(), function.clone(), payload.clone())?;

        if ! self.iodata.clone().unwrap().exchange.stage1_function_set.contains(&function) {
            return Ok(());
        }
        
        dispatch_load_state(self, spank)?;

        let mut io_data = self.iodata.clone().unwrap();
        //let output = remote_run_stage1_new(self, spank, context, function, payload)?;
        let output = run_stage1_new2(self, spank, &mut io_data)?;
        self.iodata = get_iodata_from_str(&output)?;
        info!("IODATA:\n{:#?}", self.iodata);
        //container_join_from_stage1_output(self, spank, output)?;
        container_join_from_iodata(self, spank)?;
        

        Ok(())
    }

    fn exit(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        //log_init!();
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}
        if ! self.state.enabled {
            remove_empty_log_file(self);
            return Ok(());
        }
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
        update_iodata(self, spank, context.clone(), function.clone(), payload.clone())?;

        if ! self.iodata.clone().unwrap().exchange.stage1_function_set.contains(&function) {
            return Ok(());
        }

        let mut io_data = self.iodata.clone().unwrap();
        let output = run_stage1_new2(self, spank, &mut io_data)?;
        self.iodata = get_iodata_from_str(&output)?;

        //let _ = run_stage1(self, spank, context, function, payload);
        remove_empty_log_file(self);
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
        //log_init!();
        //if !self.config.skybox_enabled {
        //    return Ok(());
        //}
        if ! self.state.enabled {
            return Ok(());
        }
        info!("TASK_EXIT");
        
        let context  = String::from("remote");
        let function = String::from("task_exit");
        let payload = self.args.payload.clone();
        update_iodata(self, spank, context.clone(), function.clone(), payload.clone())?;
        info!("IODATA:\n{:#?}", self.iodata);

        if ! self.iodata.clone().unwrap().exchange.stage1_function_set.contains(&function) {
            return Ok(());
        }

        let mut io_data = self.iodata.clone().unwrap();
        let output = run_stage1_new2(self, spank, &mut io_data)?;
        self.iodata = get_iodata_from_str(&output)?;

        //slurmstepd_task_exit(self, spank)
        //let _ = run_stage1(self, spank, context, function, payload);
        Ok(())
    }

    fn task_init_privileged(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        log_init!();
        if ! self.state.enabled {
            return Ok(());
        }
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
        update_iodata(self, spank, context.clone(), function.clone(), payload.clone())?;

        if ! self.iodata.clone().unwrap().exchange.stage1_function_set.contains(&function) {
            return Ok(());
        }

        //let _ = run_stage1(self, spank, context, function, payload);
        let mut io_data = self.iodata.clone().unwrap();
        let output = run_stage1_new2(self, spank, &mut io_data)?;
        self.iodata = get_iodata_from_str(&output)?;

        Ok(())
    }

     fn report_error(&self, spank: &mut SpankHandle, error: &dyn Error) {
        let report = format_error_chain(error);

        match spank.context().map(error_destination) {
            Ok(ErrorDestination::User) => {
                tracing::info!("{}", &report);
                spank_log_user!("{}", &report);
            }
            Ok(ErrorDestination::Tracing) => tracing::error!("{}", report),
            Err(context_error) => {
                spank_log_error!(
                    "{}; additionally failed to determine SPANK context: {}",
                    &report,
                    context_error
                );
            }
        }
    }

    fn setup(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        init_log_file(self, spank);
        set_panic_hook();
        let path = &self.log_file;

        let file = match OpenOptions::new().create(false).append(true).open(path) {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let subscriber = fmt()
            .with_writer(Mutex::new(file))
            .with_ansi(false)
            .with_target(false)
            .with_level(false)
            .with_timer(fmt::time::LocalTime::rfc_3339())
            .with_max_level(Level::INFO)
            .finish();

        tracing::subscriber::set_global_default(subscriber)?;
        Ok(())
    }
    
}
