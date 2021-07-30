use anyhow::bail;
use anyhow::Result;
use std::{path::PathBuf, process::Command};

use crate::config::{Config, FormatConfig};
use crate::log;
use crate::log::Logger;

pub fn is_lossless(format: &str) -> bool {
    match format {
        "flac" => true,
        "wav" => true,
        "aiff" => true,
        "m4a" => true,
        _ => false,
    }
}

#[derive(Clone)]
pub struct AudioFile {
    pub orig_path: PathBuf,
    pub new_path: PathBuf,
    pub converted: bool,
    pub lossless: bool,
}

impl AudioFile {
    pub fn new(orig_path: PathBuf, config: Config) -> Result<Self> {
        let ctx = ffmpeg_next::format::input(&orig_path)?;

        let mut template = config.music_files_template.clone();

        let mt = ctx.metadata();

        if mt.iter().count() == 0 {
            bail!("File contains zero metadata, it not will be converted...")
        }

        for (k, v) in mt.iter() {
            template = template.replace(format!("<{}>", k.to_lowercase()).as_str(), v);
        }

        let new_path = config.storage_path.clone().join(template);

        let lossless = is_lossless(orig_path.extension().unwrap().to_str().unwrap());

        Ok(AudioFile {
            orig_path,
            new_path,
            converted: false,
            lossless,
        })
    }

    pub fn copy(&mut self) -> Result<()> {
        println!(
            "Copying {} -> {}",
            self.orig_path.display(),
            self.new_path.display()
        );

        std::fs::copy(self.orig_path.clone(), self.new_path.clone())?;

        Ok(())
    }

    pub fn convert(&mut self, format_config: FormatConfig) -> Result<()> {
        self.new_path.set_extension(format_config.extension);

        let mut v = Vec::new();
        for t in format_config.command_line.split(" ") {
            match t {
                "<inputfile>" => {
                    v.push(self.orig_path.as_path().to_str().unwrap());
                }
                "<outputfile>" => {
                    v.push(self.new_path.as_path().to_str().unwrap());
                }
                _ => {
                    v.push(t);
                }
            }
        }

        println!(
            "Conerting {} -> {}",
            self.orig_path.display(),
            self.new_path.display()
        );
        let out = Command::new(format_config.encoder).args(v).output()?;
        log!("Stdout: {}", std::str::from_utf8(&out.stdout).unwrap_or("stdout not availble"));

        Ok(())
    }
}
