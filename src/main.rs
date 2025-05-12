use anyhow::{anyhow, ensure, Context, Result};
use argh::FromArgs;
use oib::ImageBuilder;
use path_slash::PathBufExt;
use serde::Deserialize;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::fs;

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

#[derive(FromArgs)]
/// UEFI bootable disk image creator
struct Args {
    /// output image path
    #[argh(option, short = 'o')]
    output: Option<String>,

    /// config file path
    #[argh(option, short = 'c')]
    config: Option<String>,

    /// add a file to the image (format: source:destination)
    #[argh(option, short = 'f')]
    file: Vec<String>,

    /// add a folder to the image (format: source:destination)
    #[argh(option, short = 'd')]
    dir: Vec<String>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let config_path = args.config.clone();

    // Ensure at least one configuration source is provided
    if config_path.is_none() && args.output.is_none() && args.file.is_empty() && args.dir.is_empty() {
        return Err(anyhow!("Either a config file (-c) or output path (-o) with files/folders must be provided"));
    }

    // Initialize configuration
    let mut config = if let Some(config_path) = config_path {
        // Load configuration from file
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file at '{}'", config_path))?;

        toml::from_str(&content)?
    } else {
        // Create empty config
        Config {
            output: args.output.clone().ok_or_else(|| anyhow!("Output path is required when not using a config file"))?,
            files: None,
            folders: None,
        }
    };

    // Override config with command line arguments if provided
    if let Some(output) = args.output {
        config.output = output;
    }

    // Parse file arguments
    if !args.file.is_empty() {
        let mut cli_files = Vec::new();
        for file_arg in args.file {
            let parts: Vec<&str> = file_arg.split(':').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid file format, expected source:destination in '{}'", file_arg));
            }
            cli_files.push(File {
                source: parts[0].to_string(),
                dest: parts[1].to_string(),
            });
        }

        // Combine with existing files if any
        if let Some(mut existing_files) = config.files.take() {
            existing_files.extend(cli_files);
            config.files = Some(existing_files);
        } else {
            config.files = Some(cli_files);
        }
    }

    // Parse directory arguments
    if !args.dir.is_empty() {
        let mut cli_folders = Vec::new();
        for dir_arg in args.dir {
            let parts: Vec<&str> = dir_arg.split(':').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid directory format, expected source:destination in '{}'", dir_arg));
            }
            cli_folders.push(Folder {
                source: parts[0].to_string(),
                dest: parts[1].to_string(),
            });
        }

        // Combine with existing folders if any
        if let Some(mut existing_folders) = config.folders.take() {
            existing_folders.extend(cli_folders);
            config.folders = Some(existing_folders);
        } else {
            config.folders = Some(cli_folders);
        }
    }

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
