//use std::fs::read_to_string;
//use std::path::Path;
use crate::{IOData, /*get_cache_dir_path*/};

/*
pub(crate) fn load_jobarg() -> Vec<String> {
    let jobarg = match cache2jobarg() {
        Ok(ja) => ja,
        Err(_) => vec![],
    };
    jobarg
}
*/
/*
fn cache2jobarg() -> Result<Vec<String>, String> {
    let cache_dir_path = get_cache_dir_path();
    let cache_file_path = format!("{cache_dir_path}/jobarg.json");

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

    let v: Vec<String> = match serde_json::from_str(&content) {
        Ok(vc) => vc,
        Err(e) => {
            return Err(format!("couldn't parse {cache_file_path} value as valid job arguments: {e}"));
        },
    };
    Ok(v)
}
*/

pub(crate) fn load_jobarg_from_data(data: &mut IOData) -> Vec<String> {

    let job_arg = data.exchange.job_arg.clone();
    job_arg
}
