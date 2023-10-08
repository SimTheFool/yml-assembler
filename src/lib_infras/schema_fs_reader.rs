use std::path::PathBuf;

use crate::{
    adapters::SchemaReaderPort,
    utils::result::{AppError, AppResult},
};

pub struct SchemaFSReader {
    context: PathBuf,
}
impl SchemaFSReader {
    pub fn new(path: PathBuf) -> Self {
        SchemaFSReader { context: path }
    }
}
impl SchemaReaderPort for SchemaFSReader {
    fn get_validation_schema(&self, path_str: &str) -> AppResult<serde_json::Value> {
        let path = self.context.join(format!("{path_str}"));
        let extension = path
            .extension()
            .ok_or_else(|| {
                AppError::FileSystem(format!(
                    "{path_str} has no extension, load either a json, yml or yaml file"
                ))
            })?
            .to_str()
            .ok_or_else(|| {
                AppError::FileSystem(format!(
                    "{path_str} filename probably contains invalid characters"
                ))
            })?;

        let schema = match extension {
            "json" => self.get_schema_from_json(&path)?,
            "yml" => self.get_schema_from_yml(&path)?,
            "yaml" => self.get_schema_from_yml(&path)?,
            _ => {
                return Err(AppError::FileSystem(format!(
                    "{path_str} has an invalid extension, load either a json, yml or yaml file"
                )))
            }
        };

        Ok(schema)
    }

    fn get_schema_from_json(&self, path: &PathBuf) -> AppResult<serde_json::Value> {
        println!("loading json schema: {:?}", path);
        let file = std::fs::File::open(path).map_err(AppError::other)?;
        let schema: serde_json::Value = serde_json::from_reader(file).map_err(AppError::other)?;
        Ok(schema)
    }

    fn get_schema_from_yml(&self, path: &PathBuf) -> AppResult<serde_json::Value> {
        println!("loading yml schema: {:?}", path);
        let file = std::fs::File::open(path).map_err(AppError::other)?;
        let schema: serde_json::Value = serde_yaml::from_reader(file).map_err(AppError::other)?;
        Ok(schema)
    }
}
