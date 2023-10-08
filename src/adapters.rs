use clap::ValueEnum;

use crate::utils::result::AppResult;
use std::path::PathBuf;

pub trait PartReaderPort: Send + Sync {
    fn get_value(&self, identifier: &str) -> AppResult<serde_yaml::Value>;

    fn get_filepathes_from_glob(&self, glob: &str) -> AppResult<Vec<String>>;
}

pub trait SchemaReaderPort: Send + Sync {
    fn get_validation_schema(&self, identifier: &str) -> AppResult<serde_json::Value>;

    fn get_schema_from_yml(&self, path: &PathBuf) -> AppResult<serde_json::Value>;

    fn get_schema_from_json(&self, path: &PathBuf) -> AppResult<serde_json::Value>;
}

#[derive(Debug, Clone)]
pub enum AssemblyOutputFormat {
    Yml,
    Json,
}
impl ValueEnum for AssemblyOutputFormat {
    fn from_str(input: &str, _: bool) -> Result<Self, String> {
        match input {
            "yml" => Ok(AssemblyOutputFormat::Yml),
            "json" => Ok(AssemblyOutputFormat::Json),
            _ => Err(format!("Could not parse {} as AssemblyOutputFormat", input)),
        }
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            AssemblyOutputFormat::Yml => Some(clap::builder::PossibleValue::new("yml")),
            AssemblyOutputFormat::Json => Some(clap::builder::PossibleValue::new("json")),
        }
    }

    fn value_variants<'a>() -> &'a [Self] {
        &[AssemblyOutputFormat::Yml, AssemblyOutputFormat::Json]
    }
}

pub trait AssemblyOutputPort: Send + Sync {
    fn output(
        &self,
        value: serde_yaml::Value,
        file_path: &PathBuf,
        format: &AssemblyOutputFormat,
    ) -> AppResult<()>;
}

pub trait SchemaOutputPort: Send + Sync {
    fn output(&self, value: &serde_json::Value, schema_path: &PathBuf) -> AppResult<()>;
}
