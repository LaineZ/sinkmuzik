use std::path::PathBuf;

#[derive(Clone, Copy, serde::Deserialize, PartialEq, PartialOrd)]
pub enum ConvertType {
    All,
    None,
    IfNotSame,
    OnlyLossless
}

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub storage_path: PathBuf,
    pub music_files_template: String,
    pub conversion_format: String,
    pub convert: ConvertType,
}

#[derive(Clone, serde::Deserialize)]
pub struct FormatConfig {
    pub extension: String,
    pub encoder: String,
    pub command_line: String
}
