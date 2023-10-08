use std::{path::PathBuf, sync::Arc};
use yml_assembler::{
    lib_infras::{
        assembly_in_memory_output::AssemblyIMOutput, assembly_part_fs_reader::PartFSReader,
        schema_fs_reader::SchemaFSReader, schema_in_memory_output::SchemaIMOutput,
    },
    App,
};

pub fn get_test_app() -> (App, Arc<AssemblyIMOutput>, Arc<SchemaIMOutput>) {
    let yml_reader = Arc::new(PartFSReader::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/yml_test_files")),
    ));
    let schema_reader = Arc::new(SchemaFSReader::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/yml_test_files")),
    ));
    let in_memory_assembly_output = Arc::new(AssemblyIMOutput::new());
    let in_memory_schema_output = Arc::new(SchemaIMOutput::new());

    let app = App::new(
        yml_reader,
        schema_reader,
        in_memory_assembly_output.clone(),
        in_memory_schema_output.clone(),
    );

    return (app, in_memory_assembly_output, in_memory_schema_output);
}
