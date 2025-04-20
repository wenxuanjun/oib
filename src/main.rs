use anyhow::{anyhow, ensure, Context, Result};
use oib::ImageBuilder;
use path_slash::PathBufExt;
use serde::Deserialize;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Deserialize)]
struct Config {
    output: String,
    files: Option<Vec<File>>,
    folders: Option<Vec<Folder>>,
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

    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file at '{}'", config_path))?;

    let config: Config = toml::from_str(&content)?;

    let mut files = BTreeMap::new();

    if let Some(files_config) = config.files {
        for file in files_config {
            let source = PathBuf::from(&file.source);
            ensure!(
                source.exists(),
                "Source file '{}' does not exist",
                source.display()
            );

            let dest = PathBuf::from(&file.dest);
            let dest_str = dest.to_slash().expect("Invalid UTF-8 path");
            if let Entry::Vacant(e) = files.entry(dest_str.to_string()) {
                e.insert(source);
            } else {
                println!("Skipping duplicate file: '{}'", dest_str);
            }
        }
    }

    if let Some(folders) = config.folders {
        for folder in folders {
            let src_abs = Path::new(&folder.source)
                .canonicalize()
                .with_context(|| format!("Source folder '{}' does not exist", folder.source))?;

            let dest_base = PathBuf::from(&folder.dest);
            let walker = walkdir::WalkDir::new(&src_abs)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for entry in walker {
                let source = entry.path();
                let rel_path = source.strip_prefix(&src_abs)?;

                let dest = dest_base.join(rel_path);
                let dest_str = dest.to_slash().expect("Invalid UTF-8 path");

                if let Entry::Vacant(e) = files.entry(dest_str.to_string()) {
                    e.insert(source.to_path_buf());
                } else {
                    println!("Skipping duplicate file: '{}'", dest_str);
                }
            }
        }
    }

    let output_path = PathBuf::from(config.output);
    ImageBuilder::build(files, &output_path).expect("Failed to build image");
    println!("Created bootable UEFI disk image at {:#?}", &output_path);

    Ok(())
}
