use std::path::Path;
use std::fs::{self, File};
//use std::fs::read_to_string;
use std::io::{self};
//use std::io::Write;
use url::Url;
//use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    CACHE_PATH,
    IOData,
    DataAutoUpdate,
    State, VERSION,
    expand_vars_string,
    create_dir_path,
    //get_cache_dir_path
};

/*
#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct AutoUpdate {
    pub(crate) repository: String,
    pub(crate) version: String,
}
*/

pub(crate) fn auto_update(au: DataAutoUpdate, data: &mut IOData) {
    let cur_ver = get_current_version();
    info!("Current   stage1 version is {}", cur_ver);
    info!("Requested stage1 version is {}", au.version);
    if cur_ver != au.version {
        let file_path = get_target_file_path(&au.version);
        let target_file_path = Path::new(&file_path);
        if ! target_file_path.exists() {
            match get_new_version(&au) {
                Ok(_) => {
                    if ! target_file_path.exists() {
                        info!("Failed to get requested stage1 version: {}", &au.version);
                        return;
                    }
                },
                Err(_) => {
                    info!("Failed to get requested stage1 version: {}", &au.version);
                    return;
                },
            }
        }
        info!("Requested version available at: {}", &file_path);
    }
    //au2cache(&au);
    au2data(&au, data);
}

fn get_current_version() -> String {
    return String::from(VERSION);
}

fn get_target_file_path(version: &str) -> String {
    let cache_path = expand_vars_string(CACHE_PATH.to_string()).unwrap();

    let relative_dir_path = format!("stage1/{version}/stage1");
    let target_dir_path = format!("{cache_path}/{relative_dir_path}");
    return target_dir_path;
}

fn get_source_file_path(au: &DataAutoUpdate) -> String {
    if au.repository.starts_with("http") {
        return Url::parse(&au.repository)
            .unwrap()
            .join(&format!("{}/stage1", &au.version))
            .unwrap()
            .to_string();
    } else if au.repository.starts_with("/") {
        return Path::new(&au.repository)
            .join(&format!("{}/stage1", &au.version))
            .to_string_lossy()
            .to_string();
    } else {
        return "".to_string();
    }
}

fn get_new_version(au: &DataAutoUpdate) -> Result<(),()> {
    let target_file_string = get_target_file_path(&au.version);
    let target_file_path = Path::new(&target_file_string);

    let dir_path = target_file_path.parent().unwrap();
    let _ = create_dir_path(&dir_path, 0o700);

    let file_path = get_source_file_path(au);
    info!("Requested file_path = {}", &file_path);
    if file_path == String::from("") {
        return Err(());
    } else {
        if file_path.starts_with("http") {
            let source = &Url::parse(&file_path).unwrap();
            let target = target_file_path;
            download_new_version(source, target)?;
        } else if file_path.starts_with("/") {
            let source = Path::new(&file_path);
            let target = target_file_path;
            copy_new_version(source, target)?;
        }
    }

    Ok(())
}

fn copy_new_version(source: &Path, target: &Path) -> Result<(),()> {
    info!("Attempting to copy from : {}", &source.display());

    match fs::copy(source, target) {
        Ok(_) => {
            return Ok(());
        },
        Err(_) => {
            return Err(());
        },
    }
}

fn download_new_version(source: &Url, target: &Path) -> Result<(),()> {
        info!("Attempting to download from : {}", &source.as_str());

        let mut response = match reqwest::blocking::get(source.as_str()) {
            Ok(r) => r,
            Err(_) => {
                return Err(());
            },
        };

        let mut file = match File::create(target) {
            Ok(f) => f,
            Err(_) => {
                return Err(());
            },
        };

        match io::copy(&mut response, &mut file) {
            Ok(_) => {},
            Err(_) => {
                return Err(());
            },
        }
        Ok(())
}
/*
fn au2cache(au: &DataAutoUpdate) {

    let cache_dir_path = get_cache_dir_path();
    let _ = create_dir_path(Path::new(&cache_dir_path), 0o700);
    let au_cache_file_path = format!("{cache_dir_path}/auto_update.json");

    let au_content = match serde_json::to_string(&au) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot serialize Auto Update to json");
        }
    };

    let mut file = match File::create(&au_cache_file_path) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot open {au_cache_file_path}");
        }
    };

    let _ = match file.write_all(au_content.as_bytes()) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot write to {au_cache_file_path}");
        }
    };
}
*//*
fn cache2au() -> Result<DataAutoUpdate, String> {
    let cache_dir_path = get_cache_dir_path();
    let au_cache_file_path = format!("{cache_dir_path}/auto_update.json");

    let file_path = Path::new(&au_cache_file_path);

    if ! file_path.exists() {
        return Err(format!("cannot find file {au_cache_file_path}"));
    };

    let content = match read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("cannot read file {au_cache_file_path}: {e}"));
        },
    };

    let au: DataAutoUpdate = match serde_json::from_str(&content) {
        Ok(au) => au,
        Err(e) => {
            return Err(format!("couldn't parse {au_cache_file_path} value as a valid configuration: {e}"));
        },
    };
    Ok(au)
}
*/
pub(crate) fn get_requested_exe_path(state: &State, data: &mut IOData) -> String {
    let original_exe_path = state.exe_path.clone();
    /*
    let au = match cache2au() {
        Ok(au) => au,
        Err(_) => {
            return original_exe_path;
        },
    };
    */

    let au = match data2au(data) {
        Some(au) => au,
        None => {
            return original_exe_path;
        },
    };

    let file_path = get_target_file_path(&au.version);
    let target_file_path = Path::new(&file_path);

    if ! target_file_path.exists() {
        return original_exe_path;
    }
    file_path
}

pub(crate) fn data2au(data: &mut IOData) -> Option<DataAutoUpdate> {
    let au = data.forward.clone().unwrap().auto_update.clone();
    au
}

fn au2data(au: &DataAutoUpdate, data: &mut IOData) {
    match &mut data.forward {
        None => {
            error!("data.forward shouldn;t be None at this stage");
        },
        Some(d) => {
            d.auto_update = Some(au.clone());
        },
    }
}
