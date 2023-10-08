use serde_yaml;
use yml_assembler::adapters::AssemblyOutputFormat;

pub mod test_infra;

#[derive(Debug, serde::Deserialize)]
struct DataFromYml {
    tags: Vec<String>,
    covers: Vec<CoverFromYml>,
    key_one: CompoundFromYml,
}

#[derive(Debug, serde::Deserialize)]
struct CoverFromYml {
    color: String,
    size: f64,
}

#[derive(Debug, serde::Deserialize)]
struct CompoundFromYml {
    key_two: Vec<String>,
    key_three: Vec<String>,
}

static TEST_FILE: &str = "mix_in_existing_key";

#[tokio::test]
async fn it_should_mix_on_exisiting_property() {
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

    assert_eq!(book.tags.len(), 2);
    assert!(book.tags.contains(&"childhood".to_string()));
    assert!(book.tags.contains(&"adult".to_string()));

    assert_eq!(book.covers.len(), 2);
    let green_cover = book.covers.iter().find(|c| c.color == "green").unwrap();
    assert_eq!(green_cover.size, 41.0);
    let rose_cover = book.covers.iter().find(|c| c.color == "rose").unwrap();
    assert_eq!(rose_cover.size, 15.0);
}

#[tokio::test]
async fn it_should_mix_on_compound_property() {
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

    assert_eq!(book.key_one.key_two.len(), 3);
    assert!(book.key_one.key_two.contains(&"Hi there".to_string()));
    assert!(book.key_one.key_two.contains(&"I'm a mix".to_string()));
}

#[tokio::test]
async fn it_should_mix_on_map_key() {
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

    assert_eq!(book.key_one.key_two.len(), 3);
    assert!(book
        .key_one
        .key_two
        .contains(&"I'm a second mix".to_string()));

    assert_eq!(book.key_one.key_three.len(), 1);
    assert!(book
        .key_one
        .key_three
        .contains(&"I'm a third mix".to_string()));
}
