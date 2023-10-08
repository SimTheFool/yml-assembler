use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::{
    adapters::SchemaOutputPort,
    utils::result::{AppError, AppResult},
};

pub struct SchemaIMOutput {
    pub value: Arc<RwLock<Option<serde_json::Value>>>,
}
impl SchemaIMOutput {
    pub fn new() -> Self {
        SchemaIMOutput {
            value: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_output(&self) -> AppResult<Option<serde_json::Value>> {
        let in_memory_ref = self
            .value
            .read()
            .map_err(|_| AppError::FileSystem("Cannot real schema output".to_string()))?;
        Ok(in_memory_ref.clone())
    }
}
impl SchemaOutputPort for SchemaIMOutput {
    fn output(&self, value: &serde_json::Value, _: &PathBuf) -> AppResult<()> {
        let mut in_memory_ref = self
            .value
            .write()
            .map_err(|_| AppError::FileSystem("Cannot write schema output".to_string()))?;
        *in_memory_ref = Some(serde_json::from_value(value.clone()).unwrap());
        Ok(())
    }
}
