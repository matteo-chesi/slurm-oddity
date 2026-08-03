use serde::{Deserialize, Serialize};
use slurm_spank::{SpankHandle, SpankOption};
use std::error::Error;

use crate::{SpankStage0, get_plugin_name};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Stage0Args {
    pub(crate) payload: Option<String>,
}

pub(crate) struct SpankArg {
    name: String,
    value: String,
    usage: String,
    has_arg: bool,
}

type SpankArgs = Vec<SpankArg>;

fn add_arg(mut v: SpankArgs, a: SpankArg) -> SpankArgs {
    v.push(a);
    v
}

pub(crate) fn register_plugin_args(spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
    let plug_name = get_plugin_name();

    let mut opts = vec![];
    
    opts = add_arg(
        opts,
        SpankArg {
            name: String::from("payload"),
            value: String::from("PATH"),
            usage: String::from("the path to the Environment Definition File to use."),
            has_arg: true,
        },
    );

    for opt in opts {
        let so;
        if opt.has_arg {
            so = SpankOption::new(&opt.name)
                .takes_value(&opt.value)
                .usage(format!("[{}] {}", plug_name, &opt.usage).as_str());
        } else {
            so = SpankOption::new(&opt.name)
                .usage(format!("[{}] {}", plug_name, &opt.usage).as_str());
        }
        spank.register_option(so)?;
    }
    Ok(())
}

pub(crate) fn set_arg_payload(scd: &mut SpankStage0, value: String) -> Result<(), Box<dyn Error>> {
    if value == "" {
        return Err("--payload: argument required".into());
    }
    scd.args.payload = Some(value);
    Ok(())
}

pub(crate) fn load_plugin_args(
    scd: &mut SpankStage0,
    spank: &mut SpankHandle,
) -> Result<(), Box<dyn Error>> {

    if spank.is_option_set("payload") {
        let arg_value = spank
            .get_option_value("payload")?
            .map(|s| s.to_string())
            .unwrap();
        let _ = set_arg_payload(scd, arg_value)?;
    }

    Ok(())
}
