use std::collections::HashMap;
use std::env::{remove_var, set_var};
use std::error::Error;
use std::ffi::OsStr;
use std::path::{PathBuf};
use serde::{Serialize, Deserialize};
use tracing::{info, error};

use slurm_spank::{SpankHandle};

use crate::{
    LOCAL2REMOTE_VARNAME,
    VERSION,
    SpankStage0,
    Stage0Args,
    Stage0Config,
    get_args,
    get_command_log_filepath,
    load_config,
};

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub(crate) struct IOData {
    pub(crate) source: DataSource,
    pub(crate) exchange: DataExchange,
    pub(crate) forward: Option<serde_json::Value>,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub(crate) struct DataSource {
    pub(crate) version: String,
    pub(crate) args: Stage0Args,
    pub(crate) config: Stage0Config,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub(crate) struct DataExchange {
    pub(crate) container: Option<DataContainer>,
    pub(crate) local2remote: String,
    pub(crate) payload: Option<String>,
    pub(crate) job_arg: Vec<String>,
    pub(crate) job_env: HashMap<String,String>,
    pub(crate) slurm_context: String,
    pub(crate) slurm_function: String,
    pub(crate) stage1_log_file: PathBuf,
    pub(crate) stage1_function_set: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub(crate) struct DataContainer {
    pub(crate) env: HashMap<String, String>,
    pub(crate) pid: u32,
    pub(crate) workdir: String,
}

pub(crate) fn get_iodata(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
    context: String,
    function: String,
    payload: Option<String>,
) -> Result<IOData, Box<dyn Error>> {

    let log_file = get_command_log_filepath(plugin, spank)?;
    /*
    let log_file = match context.clone().as_str() {
        "remote" => PathBuf::from(COMMAND_REMOTE_LOG_FILENAME),
        _ => PathBuf::from(COMMAND_LOCAL_LOG_FILENAME),
    };
    */

    let data_source = DataSource {
        version: VERSION.to_string(),
        args: get_args(spank)?,
        config: load_config()?,
    };

    let data_exchange = DataExchange {
        container: None,
        payload: payload.clone(), 
        job_arg: get_job_arg(spank),
        job_env: get_job_env(spank),
        slurm_context: context.clone(),
        slurm_function: function.clone(),
        stage1_log_file: log_file,
        stage1_function_set: vec![],
        local2remote: String::from(""),
    };
    
    let io_data = IOData {
        source: data_source,
        exchange: data_exchange,
        forward: None,
    };

    Ok(io_data)
}

pub(crate) fn update_iodata(
    plugin: &mut SpankStage0,
    spank: &mut SpankHandle,
    context: String,
    function: String,
    payload: Option<String>,
) -> Result<(), Box<dyn Error>> {
    /*
    if context != "local" {
        return Ok(());
    }
    */

    if plugin.iodata.is_none() {
        if function == "init_post_opt" {
            let iodata = get_iodata(plugin, spank, context.clone(), function.clone(), payload.clone())?;
            plugin.iodata = Some(iodata);
        } else {
            return Err("ERROR: IOData is not initialized!".into());
        }
    };

    let mut iodata = plugin.iodata.clone().unwrap();

    if context != iodata.exchange.slurm_context {
        return Err(format!("ERROR: Jump of context from {} to {}",
                iodata.exchange.slurm_context,
                context,
                ).into());
    }

    iodata.exchange.slurm_function = function.clone();
    iodata.exchange.payload = payload.clone();
    iodata.exchange.job_env = get_job_env(spank);

    if context == "local" && function == "init_post_opt" {
        iodata.source.args = get_args(spank)?;
    };

    plugin.iodata = Some(iodata);

    Ok(())
}

pub(crate) fn get_iodata_from_str(input: &str) -> Result<Option<IOData>, Box<dyn Error>> {

    let json: serde_json::Value = match serde_json::from_str(input) {
        Ok(j) => j,
        Err(e) => {
            error!("Error: cannot deserialize stage1 output");
            return Err(e.into());
        },
    };
    info!("OUTPUT:\n{}", serde_json::to_string_pretty(&json)?);
    let iodata: IOData = match serde_json::from_value(json) {
        Ok(j) => j,
        Err(e) => {
            error!("ERROR: {e}");
            //return Err(Box::new(e));
            // Until implemented everywhere
            return Ok(None);
        },
    };

    Ok(Some(iodata))
}

pub(crate) fn set_local2remote_env_var_from_iodata(opt: &mut Option<IOData>) {

    info!("L2R_1");
    let data = match opt {
        Some(d) => d,
        None => {
            return;
        },
    };
    
    info!("L2R_2");
    let context = &data.exchange.slurm_context;

    if context != "local" && context != "allocator" {
        return;
    }
    info!("L2R_3");

    let content = &data.exchange.local2remote;

    if content == "" {
        unsafe {
            remove_var(LOCAL2REMOTE_VARNAME);
        }
        return;
    }
    info!("L2R_4");

    unsafe {
        set_var(LOCAL2REMOTE_VARNAME, OsStr::new(content));
    }
    info!("L2R_5");
}

pub(crate) fn get_job_arg(spank: &mut SpankHandle) -> Vec<String> {
    let vec = match spank.job_argv() {
        Ok(v) => v,
        Err(_) => [].to_vec(),
    };

    let mut vc: Vec<String> = vec![];
    for v in vec.into_iter() {
        let value = v.to_string();
        vc.push(value);
    }

    return vc;
}

pub(crate) fn get_job_env(spank: &mut SpankHandle) -> HashMap<String,String> {
    let vec = match spank.job_env() {
        Ok(v) => v.clone(),
        Err(_) => [].to_vec(),
    };
    let mut h: HashMap<String,String> = HashMap::from([]);
    for v in vec.into_iter() {
        let (key, value) = v.split_once("=").unwrap();
        h.insert(key.to_string(), value.to_string());
    };

    // Integrate if needed
    if h.get("SLURM_NODEID").is_none() {
        let nodeid = match spank.job_nodeid() {
            Ok(nid) => Some(nid),
            Err(_) => None,
        };
        if nodeid.is_some() {
            let nodestr = nodeid.unwrap().to_string();
            h.insert("SLURM_NODEID".to_string(), nodestr);
        }
    }

    return h;
}
