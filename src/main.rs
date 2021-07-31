use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use walkdir::WalkDir;

use crate::{
    config::Config,
    config::{ConvertType, FormatConfig},
    file_structure::AudioFile,
    log::Logger,
    opt_parser::CmdlineOptions,
};

mod config;
mod file_structure;
mod log;
mod opt_parser;

fn main() -> Result<()> {
    Logger::new("test.log").unwrap().set_as_global();

    let run_options = CmdlineOptions::new()?;
    let main_config: Config =
        toml::from_str(&fs::read_to_string("main.toml")?).context("Cannot find main config")?;
    let encoder_config: FormatConfig = toml::from_str(&fs::read_to_string(
        PathBuf::new()
            .join("encoders/")
            .join(main_config.clone().conversion_format)
            .with_extension("toml"),
    )?)
    .context("Cannot find encoder config")?;

    ffmpeg_next::init()?;

    ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Fatal);

    println!("Welcome to sinkmuzik! Look in: {}", run_options.path);

    let mut files = Vec::new();

    for entry in WalkDir::new(run_options.path.clone()) {
        let cfg = main_config.clone();
        if let Ok(file) = AudioFile::new(entry?.into_path(), cfg) {
            files.push(file);
        }
    }

    match run_options.action {
        opt_parser::Action::Sync => {
            if !main_config.storage_path.exists() {
                fs::create_dir_all(main_config.storage_path.clone())?;
            }

            let file_failures: Vec<_> = files
                .par_iter_mut()
                .map(|file| {
                    let convert_this_file;

                    match main_config.convert {
                        config::ConvertType::All => convert_this_file = true,
                        config::ConvertType::IfNotSame => {
                            convert_this_file =
                                file.orig_path.extension() != file.new_path.extension()
                        }
                        config::ConvertType::OnlyLossless => convert_this_file = file.lossless,
                        config::ConvertType::None => convert_this_file = false,
                    }

                    if convert_this_file {
                        file.convert(encoder_config.clone())
                            .map_err(|e| e.context(format!("Cannot convert file!")))
                    } else {
                        file.new_path
                            .set_extension(file.orig_path.extension().unwrap());
                        file.copy()
                            .map_err(|e| e.context(format!("Cannot copy file!")))
                    }
                })
                .filter_map(|x| x.err())
                .collect();

            println!(
                "{} of {} files converted and saved successfully",
                files.len() - file_failures.len(),
                files.len()
            );
        }
        opt_parser::Action::Preview => {
            let mut size = 0;
            let mut count = 0;

            for file in files {
                println!(
                    "{} -> {}",
                    file.orig_path.display(),
                    file.new_path
                        .with_extension(encoder_config.extension.clone())
                        .display()
                );
                if main_config.convert == ConvertType::None {
                    size += fs::metadata(file.orig_path)?.len() / 1024 / 1024
                }
                count += 1;
            }

            println!(
                "{} file(s) will be transferred, with a total size of {} MB",
                count, size
            );
        }
    }

    Ok(())
}
