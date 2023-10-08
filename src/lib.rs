use adapters::AssemblyOutputFormat;
use jsonschema::JSONSchema;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use transformable::TransformableList;
use utils::result::AppError;
use utils::result::AppResult;
use variables::Variables;

pub mod adapters;
mod aggregator;
pub mod lib_infras;
mod mixins;
mod transformable;
pub mod utils;
mod variables;

#[derive(Clone)]
pub struct App {
    part_reader: Arc<dyn adapters::PartReaderPort>,
    schema_reader: Arc<dyn adapters::SchemaReaderPort>,
    assembly_output: Arc<dyn adapters::AssemblyOutputPort>,
    schema_output: Arc<dyn adapters::SchemaOutputPort>,
}

impl App {
    pub fn new(
        yml_reader: Arc<dyn adapters::PartReaderPort>,
        schema_reader: Arc<dyn adapters::SchemaReaderPort>,
        assembly_output: Arc<dyn adapters::AssemblyOutputPort>,
        schema_output: Arc<dyn adapters::SchemaOutputPort>,
    ) -> Self {
        Self {
            part_reader: yml_reader,
            schema_reader,
            assembly_output,
            schema_output,
        }
    }

    pub fn compile_and_validate_yml(
        &self,
        yml_id: &str,
        schema_id: Option<&str>,
        variables: Option<HashMap<String, String>>,
        format: &AssemblyOutputFormat,
    ) -> AppResult<()> {
        let mut aggregator = aggregator::YmlAggregator::new(Arc::clone(&self.part_reader));

        let variables: Variables = variables.unwrap_or(HashMap::new()).into();
        let yml = aggregator.load(yml_id, &variables)?;
        let mixins = aggregator.mixins;
        let yml = mixins.inject(&yml)?;

        let mut list = TransformableList::try_from(yml)?;
        list.transform()?;
        let yml: serde_yaml::Value = list.try_into()?;

        let schema_json = match schema_id {
            Some(schema_id) => {
                let yml_json_representation =
                    serde_json::to_value(&yml).map_err(AppError::other)?;
                let schema_json = self.schema_reader.get_validation_schema(schema_id)?;
                let validator = JSONSchema::compile(&schema_json)
                    .map_err(|e| AppError::ValidateYml(format!("Schema is not valid: {}", e)))?;
                validator.validate(&yml_json_representation).map_err(|e| {
                    let str_errors = e
                        .into_iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>()
                        .join("\n");

                    AppError::ValidateYml(format!("Generated yml is not valid: {}", str_errors))
                })?;

                Some(schema_json)
            }
            None => None,
        };

        self.assembly_output
            .output(yml, &PathBuf::from(yml_id), format)?;

        if let Some(schema_json) = schema_json {
            self.schema_output
                .output(&schema_json, &PathBuf::from(schema_id.unwrap()))?;
        }

        Ok(())
    }
}
