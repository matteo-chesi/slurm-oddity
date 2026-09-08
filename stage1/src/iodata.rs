use std::collections::HashMap;
use std::error::Error;
use std::io::{Write, stdin, stdout};
use std::path::{PathBuf};
use serde::{Serialize, Deserialize};

use raster::{Config};

#[derive(Deserialize, Serialize, Default, Debug)]
pub(crate) struct IOData {
    pub(crate) source: DataSource,
    pub(crate) exchange: DataExchange,
    pub(crate) forward: Option<DataForward>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub(crate) struct DataSource {
    pub(crate) version: String,
    pub(crate) args: Stage0Args,
    pub(crate) config: Stage0Config,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub(crate) struct DataExchange {
    pub(crate) container: Option<DataContainer>,
    pub(crate) local2remote: String,
    pub(crate) job_arg: Vec<String>,
    pub(crate) job_env: HashMap<String,String>,
    pub(crate) payload: Option<String>,
    pub(crate) slurm_context: String,
    pub(crate) slurm_function: String,
    pub(crate) stage1_log_file: PathBuf,
    pub(crate) stage1_function_set: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub(crate) struct DataForward {
    pub(crate) config: Config,
    pub(crate) auto_update: Option<DataAutoUpdate>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub(crate) struct DataContainer {
    pub(crate) env: HashMap<String, String>,
    pub(crate) pid: u32,
    pub(crate) workdir: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub(crate) struct DataAutoUpdate {
    pub(crate) repository: String,
    pub(crate) version: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub(crate) struct Stage0Args {
    pub(crate) payload: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Stage0Config {
    pub stage1_user_path: String,
    pub stage1_system_path: String,
}

pub(crate) fn get_iodata_from_stdin() -> Result<IOData, Box<dyn Error>> {
    
    let mut buffer = String::new();
    let _ = stdin().read_line(&mut buffer)?;
    let input = buffer.trim();
    let data: IOData = serde_json::from_str(&input)?;

    Ok(data)
}

pub(crate) fn send_iodata_to_stdout(iodata: &IOData) -> Result<(), Box<dyn Error>> {
    let json_string = serde_json::to_string(iodata)?; 
    println!("{json_string}");
    let _ = stdout().flush();
    Ok(())
}
