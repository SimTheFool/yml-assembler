use crate::{
    adapters::PartReaderPort,
    utils::result::{AppError, AppResult},
};
use glob::glob;
use std::{collections::HashMap, path::PathBuf, sync::RwLock};

pub struct PartFSReader {
    context: PathBuf,
    read_cache: RwLock<HashMap<String, serde_yaml::Value>>,
}
impl PartFSReader {
    pub fn new(path: PathBuf) -> Self {
        PartFSReader {
            context: path,
            read_cache: RwLock::new(HashMap::new()),
        }
    }
}
impl PartReaderPort for PartFSReader {
    fn get_filepathes_from_glob(&self, glob_str: &str) -> AppResult<Vec<String>> {
        let absolute_context = self.context.canonicalize().map_err(AppError::other)?;
        let glob_path = absolute_context.join(glob_str).with_extension("pyml");
        let glob_path = glob_path
            .to_str()
            .ok_or_else(|| AppError::FileSystem("Could not convert path to str".to_string()))?;

        let filepathes = glob(glob_path).map_err(AppError::other)?;
        let filepathes = filepathes
            .map(|filepath| filepath.map_err(AppError::other))
            .collect::<Result<Vec<PathBuf>, AppError>>()?;

        let filepathes = filepathes
            .iter()
            .map(|filepath| {
                let path = filepath
                    .strip_prefix(&absolute_context)
                    .map_err(AppError::other)?;

                let path = path.with_extension("");

                let path_str = path.to_str().ok_or_else(|| {
                    AppError::FileSystem("Could not convert path to str".to_string())
                })?;

                Ok(path_str.to_string())
            })
            .collect::<AppResult<Vec<String>>>()?;

        Ok(filepathes)
    }

    fn get_value(&self, identifier: &str) -> AppResult<serde_yaml::Value> {
        let mut cache = self.read_cache.write().map_err(|e| {
            AppError::FileSystem(format!("Could not write to cache: {}", e.to_string()))
        })?;
        let cached_value = cache.get(identifier);

        match cached_value {
            Some(value) => {
                println!("reading from cache: {}", identifier);
                Ok(value.clone())
            }
            None => {
                println!("reading: {}", identifier);
                let path = self.context.join(format!("{identifier}.pyml"));
                let file = std::fs::read_to_string(path).map_err(AppError::other)?;
                let yml: serde_yaml::Value =
                    serde_yaml::from_str(&file).map_err(AppError::other)?;

                cache.insert(identifier.to_string(), yml.clone());
                Ok(yml)
            }
        }
    }
}
