use oib::{Files, ImageBuilder};
use serde::Deserialize;
use std::env;
use std::path::{absolute, PathBuf};

#[derive(Deserialize)]
struct Config {
    output: String,
    files: Vec<File>,
    folders: Vec<Folder>,
}

#[derive(Deserialize)]
struct File {
    source: String,
    dest: String,
}

#[derive(Deserialize)]
struct Folder {
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

    let mut files: Files = config
        .files
        .into_iter()
        .map(|file| (file.dest, PathBuf::from(file.source)))
        .collect();

    config.folders.into_iter().for_each(|folder| {
        for entry in walkdir::WalkDir::new(folder.source.clone()) {
            if let Ok(file) = entry {
                let file_source = file.path();

                let folder_path = absolute(folder.source.clone()).unwrap();

                let file_dest = PathBuf::from(folder.dest.clone())
                    .join(file_source.strip_prefix(folder_path).unwrap());
                files.insert(
                    file_dest.to_str().unwrap().to_string(),
                    file_source.to_path_buf(),
                );
            }
        }
    });

    let output_path = PathBuf::from(config.output);
    ImageBuilder::build(files, &output_path).expect("Failed to build UEFI disk image");
    println!("Created bootable UEFI disk image at {:#?}", &output_path);
}
