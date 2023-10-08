use std::path::PathBuf;

use crate::{adapters::SchemaOutputPort, utils::result::AppResult};

pub struct SchemaFSOutput {
    context: PathBuf,
}

impl SchemaFSOutput {
    pub fn new(path: PathBuf) -> Self {
        SchemaFSOutput { context: path }
    }
}

impl SchemaOutputPort for SchemaFSOutput {
    fn output(&self, value: &serde_json::Value, schema_path: &PathBuf) -> AppResult<()> {
        let outschema_path = PathBuf::from(&self.context)
            .join(&schema_path)
            .with_extension("json");

        let outschema_parent = outschema_path.parent().ok_or_else(|| {
            anyhow::anyhow!(format!(
                "Could not get parent directory of {}",
                outschema_path.display()
            ))
        })?;

        if !outschema_parent.exists() {
            std::fs::create_dir_all(&outschema_parent).map_err(|e| {
                anyhow::anyhow!(format!("Could not create output directory: {}", e))
            })?;
        }

        std::fs::write(
            outschema_path,
            serde_json::to_string_pretty(value)
                .map_err(|e| anyhow::anyhow!(format!("Could not serialize file: {}", e)))?,
        )
        .map_err(|e| {
            anyhow::anyhow!(format!("Could not write schema to output directory: {}", e))
        })?;

        Ok(())
    }
}
