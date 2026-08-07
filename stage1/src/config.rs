use std::ffi::OsStr;

use raster::{Config, load_config as raster_load_config};

pub(crate) fn load_config() -> Config {
    let mut config = match env2config() {
        Ok(cfg) => cfg,
        Err(_) => {
            match raster_load_config() {
                Ok(cfg) => cfg,
                Err(_) => {
                    panic!("Cannot load configuration");
                },
            }
        },
    };

    if config.launch_control_repository == "" {
        config2env(&config);
        return config;
    } else {
        config = load_config_from_launch_control(&config);
    }

    config2env(&config);
    
    config
}

fn config2env(config:  &Config) {
    let key = "SLURM_STAGE1_CONFIG";
    let value = match serde_json::to_string(config) {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot serialize config to json");
        }
    };
    unsafe {
        std::env::set_var(key, OsStr::new(&value));
    }
}

fn env2config() -> Result<Config, String> {
    let key = "SLURM_STAGE1_CONFIG";
    let value = match std::env::var(key) {
        Ok(s) => s,
        Err(e) => {
            return Err(format!("couldn't find/read {key}: {e}"));
        },
    };

    let config: Config = match serde_json::from_str(&value.as_str()) {
        Ok(cfg) => cfg,
        Err(e) => {
            return Err(format!("couldn't parse {key} value as a valid configuration: {e}"));
        },
    };
    Ok(config)
}

fn load_config_from_launch_control(old_config: &Config) -> Config {
    let mut config = old_config.clone();
    config
}
