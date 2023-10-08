use serde_yaml;
use yml_assembler::adapters::AssemblyOutputFormat;

pub mod test_infra;

#[derive(Debug, serde::Deserialize)]
struct DataFromYml {
    content: String,
    content_bis: String,
    chapter: Vec<u8>,
}

static TEST_FILE: &str = "deep_replace_variables";

#[tokio::test]
async fn it_should_deep_replace_variables() {
    let (app, assembly_output, _) = test_infra::get_test_app();
    app.compile_and_validate_yml(TEST_FILE, None, None, &AssemblyOutputFormat::Yml)
        .unwrap();
    let yml = assembly_output
        .get_yml_output()
        .unwrap()
        .get(TEST_FILE)
        .unwrap()
        .clone();
    let book: DataFromYml = serde_yaml::from_value(yml).unwrap();

    assert_eq!(book.content, "Some car crashed".to_string());
    assert_eq!(book.content_bis, "Some car crashed".to_string());
    assert!(book.chapter.contains(&2));
}

#[tokio::test]
async fn it_should_work_well_with_mixin() {
    let (app, assembly_output, _) = test_infra::get_test_app();
    app.compile_and_validate_yml(TEST_FILE, None, None, &AssemblyOutputFormat::Yml)
        .unwrap();
    let yml = assembly_output
        .get_yml_output()
        .unwrap()
        .get(TEST_FILE)
        .unwrap()
        .clone();
    let book: DataFromYml = serde_yaml::from_value(yml).unwrap();

    assert!(book.chapter.contains(&3));
    assert!(book.chapter.contains(&16));
}
