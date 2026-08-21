use std::env;
use std::collections::HashMap;
use std::ffi::{OsString};
use std::fs::File;
use std::fs::{read_to_string};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use serde_json::{map::Entry, Value};
use url::Url;
use users::{get_current_groupname};
use regex::Regex;

use raster::{Config, config::ConfigHooks as ConfigHooks, load_config as raster_load_config};
use crate::{AutoUpdate, auto_update, create_dir_path, get_cache_dir_path, log};

pub(crate) fn load_config() -> Config {
    //print_home();
    let mut config = match cache2config() {
        Ok(cfg) => {
            return cfg;
        },
        Err(_) => {
            match raster_load_config() {
                Ok(cfg) => cfg,
                Err(_) => {
                    panic!("Cannot load configuration");
                },
            }
        },
    };

    config = load_config_from_launch_control(&config);
    config2cache(&config);
    config
}

fn config2cache(config: &Config) {

    let cache_dir_path = get_cache_dir_path();
    let _ = create_dir_path(Path::new(&cache_dir_path));

    let config_cache_file_path = format!("{cache_dir_path}/config.json");

    let config_content = match serde_json::to_string(&config) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot serialize config to json");
        }
    };

    let mut file = match File::create(&config_cache_file_path) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot open {config_cache_file_path}");
        }
    };

    let _ = match file.write_all(config_content.as_bytes()) {
        Ok(f) => f,
        Err(_) => {
            panic!("Cannot write to {config_cache_file_path}");
        }
    };
}

fn cache2config() -> Result<Config, String> {
    let cache_dir_path = get_cache_dir_path();
    let mut config_cache_file_path = format!("{cache_dir_path}/config.json");

    // for task_init, remove _task_### suffix
    let re = Regex::new(r"_task_\d+").unwrap();
    config_cache_file_path = re.replace(&config_cache_file_path, "").to_string();

    log(&format!("Looking for config at: {}", &config_cache_file_path));

    let file_path = Path::new(&config_cache_file_path);

    if ! file_path.exists() {
        return Err(format!("cannot find file {config_cache_file_path}"));
    };

    let content = match read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            return Err(format!("cannot read file {config_cache_file_path}: {e}"));
        },
    };

    let config: Config = match serde_json::from_str(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            return Err(format!("couldn't parse {config_cache_file_path} value as a valid configuration: {e}"));
        },
    };
    Ok(config)
}

fn load_config_from_launch_control(old_config: &Config) -> Config {
    let mut config = old_config.clone();

    // check if launch control is set
    if config.launch_control_repository == "" {
        return config;
    }

    // check if launch control is a valid url
    let url = match Url::parse(&config.launch_control_repository) {
        Ok(u) => u,
        Err(_) => {
            return config;
        },
    };
    let query_url = url.join("/get_config").unwrap();

    // Send PUT request & Wait for response
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();

    let query_input = build_query_input();
    let query_output = client.post(query_url.clone()).json(&query_input).send();
    log(&format!("Query Launch Control at: {}", query_url.as_str()));
    log(&format!("Query Input:\n{}", serde_json::to_string_pretty(&query_input).unwrap()));

    let output = match query_output {
        Ok(response) => response,
        Err(_) => {
            return config;
        },
    };

    // Parse response
    //let json: Value = match serde_json::from_str(output) {
    let mut json: Value = match output.json() {
        Ok(s) => s,
        Err(_) => {
            return config;
        }
    };
    log(&format!("Launch Control Response:\n{}", serde_json::to_string_pretty(&json).unwrap()));

    config = update_config_from_json(&mut config, &mut json);

    match get_auto_update_data_from_json(&json) {
        Some(au) => auto_update(au),
        None => {},
    };

    config
}

fn build_query_input() -> HashMap<String, String> {
    let mut input = HashMap::new();

    let unknown_job = String::from("0");
    let unknown = String::from("UNKNOWN");
    let job = env::var("SLURM_JOB_ID").unwrap_or(unknown_job);
    let system = env::var("CLUSTER_NAME").unwrap_or(unknown.clone());

    let account;
    let remote_account = env::var("SLURM_JOB_ACCOUNT").unwrap_or(unknown.clone());

    if remote_account != "UNKNOWN" {
        account = remote_account;
    } else {
        account = get_current_groupname()
            .unwrap_or(OsString::from("UNKNOWN"))
            .into_string()
            .unwrap_or(unknown.clone());
    }

    let user;
    let remote_user = env::var("SLURM_JOB_USER").unwrap_or(unknown.clone());

    if remote_user != "UNKNOWN" {
        user = remote_user;
    } else {
        user = env::var("USER").unwrap_or(unknown.clone());
    }

    input.insert("job".to_string(), job.to_string());
    input.insert("system".to_string(), system.to_string());
    input.insert("account".to_string(), account.to_string());
    input.insert("user".to_string(), user.to_string());

    return input;
}

fn update_config_from_json(old_config: &mut Config, json: &mut Value) -> Config {

    let mut config = old_config.clone();

    let root = match json.as_object_mut() {
        Some(obj) => obj,
        None => {
            return config;
        },
    };

    let config_val = root.entry("config".to_string());
    match config_val {
        Entry::Occupied(mut entry) => {
            update_config_from_value(&mut config, &mut entry.get_mut());
        },
        Entry::Vacant(_) => {},
    };

    config
}

fn update_config_from_value(config: &mut Config, value: &mut Value) {

    // ensure it is an object
    let config_obj = match value.as_object() {
            Some(cfg) => cfg,
            None => {
                return;
            },
    };

    // Loop through config keys
    for (key, value) in config_obj.into_iter() {
        match key.as_str() {
            "edf_system_search_path" => {
                if value.is_string() {
                    config.edf_system_search_path = String::from(value.as_str().unwrap());
                }
            },
            "hooks" => {
                if value.is_object() {
                    update_confighooks_from_value(&mut config.hooks, &value);
                }
            },
            "launch_control_repository" => {
                if value.is_string() {
                    config.launch_control_repository = String::from(value.as_str().unwrap());
                }
            },
            "launch_control_cache_path" => {
                if value.is_string() {
                    config.launch_control_cache_path = String::from(value.as_str().unwrap());
                }
            },
            "parallax_imagestore" => {
                if value.is_string() {
                    config.parallax_imagestore = String::from(value.as_str().unwrap());
                }
            },
            "parallax_imagestore_keepalive" => {
                if value.is_boolean() {
                    config.parallax_imagestore_keepalive = value.as_bool().unwrap();
                }
            },
            "parallax_mount_program" => {
                if value.is_string() {
                    config.parallax_mount_program = String::from(value.as_str().unwrap());
                }
            },
            "parallax_path" => {
                if value.is_string() {
                    config.parallax_path = String::from(value.as_str().unwrap());
                }
            },
            "parallax_mp_uid" => {
                if value.is_number() {
                    if value.is_u64() {
                        match value.as_u64() {
                            Some(num) if num <= u32::MAX as u64 => {
                                config.parallax_mp_uid = num as u32;
                            },
                            _ => {},
                        }
                    }
                }
            },
            "parallax_mp_gid" => {
                if value.is_number() {
                    if value.is_u64() {
                        match value.as_u64() {
                            Some(num) if num <= u32::MAX as u64 => {
                                config.parallax_mp_gid = num as u32;
                            },
                            _ => {},
                        }
                    }
                }
            },
            "parallax_mp_logfile" => {
                if value.is_string() {
                    config.parallax_mp_logfile = String::from(value.as_str().unwrap());
                }
            },
            "parallax_mp_squashfuse_path" => {
                if value.is_string() {
                    config.parallax_mp_squashfuse_path = String::from(value.as_str().unwrap());
                }
            },
            "perfmon" => {
                if value.is_boolean() {
                    config.perfmon = value.as_bool().unwrap();
                }
            },
            "podman_module" => {
                if value.is_string() {
                    config.podman_module = String::from(value.as_str().unwrap());
                }
            },
            "podman_path" => {
                if value.is_string() {
                    config.podman_path = String::from(value.as_str().unwrap());
                }
            },
            "podman_tmp_path" => {
                if value.is_string() {
                    config.podman_tmp_path = String::from(value.as_str().unwrap());
                }
            },
            "runtime_path" => {
                if value.is_string() {
                    config.runtime_path = String::from(value.as_str().unwrap());
                }
            },
            "skybox_enabled" => {
                if value.is_boolean() {
                    config.skybox_enabled = value.as_bool().unwrap();
                }
            },
            "tracking_enabled" => {
                if value.is_boolean() {
                    config.tracking_enabled = value.as_bool().unwrap();
                }
            },
            "tracking_tool" => {
                if value.is_string() {
                    config.tracking_tool = String::from(value.as_str().unwrap());
                }
            },
            _ => {},
        }
    }
}

fn update_confighooks_from_value(cfghooks: &mut ConfigHooks, value: &Value) {

    // ensure it is an object
    let hooks_obj = match value.as_object() {
            Some(obj) => obj,
            None => {
                return;
            },
    };

    // Loop through confighooks keys
    for (key, value) in hooks_obj.into_iter() {
        match key.as_str() {
            "parallax_imagestore_create" => {
                if value.is_string() {
                    cfghooks.parallax_imagestore_create = String::from(value.as_str().unwrap());
                }
            },
            _ => {},
        }
    }
}

fn get_auto_update_data_from_json(json: &Value) -> Option<AutoUpdate> {

    let stage1_repository;
    let stage1_version;

    let root = match json.as_object() {
        Some(obj) => obj,
        None => {
            return None;
        },
    };

    let repo_val = root.get("stage1_repository");
    match repo_val {
        Some(value) => {
            if value.is_string() {
                stage1_repository = value.as_str().unwrap().to_string();
            } else {
                return None;
            }
        },
        None => {
            return None;
        },
    };

    let ver_val = root.get("stage1_version");
    match ver_val {
        Some(value) => {
            if value.is_string() {
                stage1_version = value.as_str().unwrap().to_string();
            } else {
                return None;
            }
        },
        None => {
            return None;
        },
    };

    Some(AutoUpdate {
        repository: stage1_repository,
        version: stage1_version,
    })
}

/*
fn print_home() {
    let home_var = env::var("HOME").unwrap_or("${HOME}".to_string());
    log(&format!("HOME = {}", &home_var));
}
*/
