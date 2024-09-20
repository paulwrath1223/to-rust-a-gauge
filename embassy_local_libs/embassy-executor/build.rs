use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use std::{env, fs};

#[path = "./build_common.rs"]
mod common;

static CONFIGS: &[(&str, usize)] = &[
    // BEGIN AUTOGENERATED CONFIG FEATURES
    // Generated by gen_config.py. DO NOT EDIT.
    ("TASK_ARENA_SIZE", 4096),
    // END AUTOGENERATED CONFIG FEATURES
];

struct ConfigState {
    value: usize,
    seen_feature: bool,
    seen_env: bool,
}

fn main() {
    let crate_name = env::var("CARGO_PKG_NAME")
        .unwrap()
        .to_ascii_uppercase()
        .replace('-', "_");

    // only rebuild if build.rs changed. Otherwise Cargo will rebuild if any
    // other file changed.
    println!("cargo:rerun-if-changed=build.rs");

    // Rebuild if config envvar changed.
    for (name, _) in CONFIGS {
        println!("cargo:rerun-if-env-changed={crate_name}_{name}");
    }

    let mut configs = HashMap::new();
    for (name, default) in CONFIGS {
        configs.insert(
            *name,
            ConfigState {
                value: *default,
                seen_env: false,
                seen_feature: false,
            },
        );
    }

    let prefix = format!("{crate_name}_");
    for (var, value) in env::vars() {
        if let Some(name) = var.strip_prefix(&prefix) {
            let Some(cfg) = configs.get_mut(name) else {
                panic!("Unknown env var {name}")
            };

            let Ok(value) = value.parse::<usize>() else {
                panic!("Invalid value for env var {name}: {value}")
            };

            cfg.value = value;
            cfg.seen_env = true;
        }

        if let Some(feature) = var.strip_prefix("CARGO_FEATURE_") {
            if let Some(i) = feature.rfind('_') {
                let name = &feature[..i];
                let value = &feature[i + 1..];
                if let Some(cfg) = configs.get_mut(name) {
                    let Ok(value) = value.parse::<usize>() else {
                        panic!("Invalid value for feature {name}: {value}")
                    };

                    // envvars take priority.
                    if !cfg.seen_env {
                        if cfg.seen_feature {
                            panic!("multiple values set for feature {}: {} and {}", name, cfg.value, value);
                        }

                        cfg.value = value;
                        cfg.seen_feature = true;
                    }
                }
            }
        }
    }

    let mut data = String::new();

    for (name, cfg) in &configs {
        writeln!(&mut data, "pub const {}: usize = {};", name, cfg.value).unwrap();
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("config.rs").to_string_lossy().to_string();
    fs::write(out_file, data).unwrap();

    let mut rustc_cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut rustc_cfgs);
}
