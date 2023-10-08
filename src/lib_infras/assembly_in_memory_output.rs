use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::{
    adapters::{AssemblyOutputFormat, AssemblyOutputPort},
    utils::result::{AppError, AppResult},
};

pub struct AssemblyIMOutput {
    pub value_yml: Arc<RwLock<HashMap<String, serde_yaml::Value>>>,
    pub value_json: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}
impl AssemblyIMOutput {
    pub fn new() -> Self {
        AssemblyIMOutput {
            value_yml: Arc::new(RwLock::new(HashMap::new())),
            value_json: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn get_yml_output(&self) -> AppResult<HashMap<String, serde_yaml::Value>> {
        let in_memory_ref = self
            .value_yml
            .read()
            .map_err(|_| AppError::FileSystem("Cannot real yml output".to_string()))?;
        Ok(in_memory_ref.clone())
    }
    pub fn get_json_output(&self) -> AppResult<HashMap<String, serde_json::Value>> {
        let in_memory_ref = self
            .value_json
            .read()
            .map_err(|_| AppError::FileSystem("Cannot real json output".to_string()))?;
        Ok(in_memory_ref.clone())
    }
}
impl AssemblyOutputPort for AssemblyIMOutput {
    fn output(
        &self,
        value: serde_yaml::Value,
        key: &PathBuf,
        format: &AssemblyOutputFormat,
    ) -> AppResult<()> {
        match format {
            AssemblyOutputFormat::Yml => {
                let mut in_memory_ref = self
                    .value_yml
                    .write()
                    .map_err(|_| AppError::FileSystem("Cannot write yml output".to_string()))?;
                in_memory_ref.insert(key.to_str().unwrap().to_string(), value.clone());
            }
            AssemblyOutputFormat::Json => {
                let mut in_memory_ref = self
                    .value_json
                    .write()
                    .map_err(|_| AppError::FileSystem("Cannot write json output".to_string()))?;
                let json = serde_json::to_value(value).map_err(|e| {
                    anyhow::anyhow!(format!("Could not transform yml to json: {}", e))
                })?;
                in_memory_ref.insert(key.to_str().unwrap().to_string(), json);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    #[test]
    fn it_should_set_out_in_hashmap_entry() {
        use crate::adapters::AssemblyOutputPort;
        let assembly_output = super::AssemblyIMOutput::new();

        let value_1 = serde_yaml::Value::String(
            r#"
            key1: value1
            key2: value2
        "#
            .to_string(),
        );
        let key_1 = PathBuf::from("folder1/file1".to_string());

        let value_2 = serde_yaml::Value::String(
            r#"
            - a1
            - a2
        "#
            .to_string(),
        );

        let key_2 = PathBuf::from("folder2/file2".to_string());

        assembly_output
            .output(
                value_1.clone(),
                &key_1,
                &crate::adapters::AssemblyOutputFormat::Yml,
            )
            .unwrap();
        assembly_output
            .output(
                value_2.clone(),
                &key_2,
                &crate::adapters::AssemblyOutputFormat::Yml,
            )
            .unwrap();

        let output = assembly_output.get_yml_output().unwrap();

        let entry1 = output.get(key_1.to_str().unwrap()).unwrap();
        let entry2 = output.get(key_2.to_str().unwrap()).unwrap();

        assert_eq!(entry1, &value_1);
        assert_eq!(entry2, &value_2);
    }
}
