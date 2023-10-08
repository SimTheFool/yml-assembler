use std::path::PathBuf;

use crate::{
    adapters::{AssemblyOutputFormat, AssemblyOutputPort},
    utils::result::AppResult,
};

pub struct AssemblyFSOutput {
    context: PathBuf,
}

impl AssemblyFSOutput {
    pub fn new(path: PathBuf) -> Self {
        AssemblyFSOutput { context: path }
    }
}

pub enum OutputFormat {
    Yml,
    Json,
}

impl AssemblyOutputPort for AssemblyFSOutput {
    fn output(
        &self,
        value: serde_yaml::Value,
        file_path: &PathBuf,
        format: &AssemblyOutputFormat,
    ) -> AppResult<()> {
        let outfile_path = PathBuf::from(&self.context).join(file_path);

        let outfile_parent = outfile_path.parent().ok_or_else(|| {
            anyhow::anyhow!(format!(
                "Could not get parent directory of {}",
                outfile_path.display()
            ))
        })?;

        if !outfile_parent.exists() {
            std::fs::create_dir_all(&outfile_parent).map_err(|e| {
                anyhow::anyhow!(format!("Could not create output directory: {}", e))
            })?;
        }

        let (value_str, extension) = match format {
            AssemblyOutputFormat::Yml => {
                let str = serde_yaml::to_string(&value)
                    .map_err(|e| anyhow::anyhow!(format!("Could not serialize yml file: {}", e)))?;
                let extension = "yml";
                (str, extension)
            }
            AssemblyOutputFormat::Json => {
                let json = serde_json::to_value(value).map_err(|e| {
                    anyhow::anyhow!(format!("Could not transform yml to json: {}", e))
                })?;
                let str = serde_json::to_string_pretty(&json).map_err(|e| {
                    anyhow::anyhow!(format!("Could not serialize json file: {}", e))
                })?;
                let extension = "json";
                (str, extension)
            }
        };

        let outfile_path = outfile_path.with_extension(extension);
        std::fs::write(outfile_path, value_str).map_err(|e| {
            anyhow::anyhow!(format!("Could not write file to output directory: {}", e))
        })?;

        Ok(())
    }
}
