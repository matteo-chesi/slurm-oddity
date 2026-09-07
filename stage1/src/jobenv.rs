use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use crate::{IOData, get_cache_dir_path};

pub(crate) fn load_jobenv() -> HashMap<String, String> {
    let jobenv = match cache2jobenv() {
        Ok(je) => je,
        Err(_) => HashMap::from([]),
    };
    jobenv
}

fn cache2jobenv() -> Result<HashMap<String, String>, String> {
    let cache_dir_path = get_cache_dir_path();
    let cache_file_path = format!("{cache_dir_path}/jobenv.json");

    let file_path = Path::new(&cache_file_path);

    if ! file_path.exists() {
        return Err(format!("cannot find file {cache_file_path}"));
    };

    let content = match read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("cannot read file {cache_file_path}: {e}"));
        },
    };

    let h: HashMap<String, String> = match serde_json::from_str(&content) {
        Ok(hm) => hm,
        Err(e) => {
            return Err(format!("couldn't parse {cache_file_path} value as a valid job environment: {e}"));
        },
    };
    Ok(h)
}

pub(crate) fn load_jobenv_from_data(data: &mut IOData) -> HashMap<String,String> {

    let job_env = data.exchange.job_env.clone();
    job_env
}
