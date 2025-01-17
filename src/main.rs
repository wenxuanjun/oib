use oib::{Files, ImageBuilder};
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Config {
    output: String,
    files: Vec<File>,
}

#[derive(Deserialize)]
struct File {
    source: String,
    dest: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <config_path>", args[0]);
        std::process::exit(1);
    }

    let config_path = &args[1];
    let content = match std::fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            std::process::exit(1);
        }
    };

    let config: Config = match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to parse config: {}", e);
            std::process::exit(1);
        }
    };

    let files: Files = config
        .files
        .into_iter()
        .map(|file| (file.dest, PathBuf::from(file.source)))
        .collect();

    let output_path = PathBuf::from(config.output);
    ImageBuilder::build(files, &output_path).expect("Failed to build UEFI disk image");
    println!("Created bootable UEFI disk image at {:#?}", &output_path);
}
