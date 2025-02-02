use anyhow::{anyhow, Result};
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

fn main() -> Result<()> {
    let config_path = env::args().nth(1).ok_or_else(|| {
        let exe_path = env::args().next().unwrap();
        anyhow!("Usage: {exe_path} <config_path>")
    })?;

    let content = std::fs::read_to_string(&config_path)?;
    let config = toml::from_str::<Config>(&content)?;

    let mut files: Files = config
        .files
        .into_iter()
        .map(|file| (file.dest, PathBuf::from(file.source)))
        .collect();

    for folder in config.folders {
        let folder_path = absolute(&folder.source)?;
        let to_add = walkdir::WalkDir::new(&folder.source)
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let source = entry.path();
                let abs_path = source.strip_prefix(&folder_path).ok()?;
                let dest = PathBuf::from(&folder.dest).join(abs_path);
                Some((dest.to_str()?.to_string(), source.to_path_buf()))
            });
        files.extend(to_add);
    }

    let output_path = PathBuf::from(config.output);
    ImageBuilder::build(files, &output_path).expect("Failed to build image");
    println!("Created bootable UEFI disk image at {:#?}", &output_path);

    Ok(())
}
