use std::collections::HashMap;

use serde_yaml::{self, Value};
use yml_assembler::adapters::AssemblyOutputFormat;

pub mod test_infra;

static TEST_FILE: &str = "labeled_transform";

#[tokio::test]
async fn it_should_apply_labeled_transform_in_abcd_order() {
    let mut variables = HashMap::new();
    variables.insert("T_LAYER".to_string(), "t30".to_string());

    let (app, assembly_output, _) = test_infra::get_test_app();
    app.compile_and_validate_yml(TEST_FILE, None, Some(variables), &AssemblyOutputFormat::Yml)
        .unwrap();
    let yml = assembly_output
        .get_yml_output()
        .unwrap()
        .get(TEST_FILE)
        .unwrap()
        .clone();

    match yml {
        Value::Mapping(m) => {
            let a = m.get(&Value::String("a".to_string()));
            assert_eq!(a.unwrap().as_i64().unwrap(), 937);
        }
        _ => panic!("Yml should be a mapping"),
    };
}
