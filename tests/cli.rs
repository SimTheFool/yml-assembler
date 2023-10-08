use assert_cmd::prelude::{OutputAssertExt, *};
use predicates::prelude::*;
use serial_test::serial;
use std::{collections::HashMap, fs, path::PathBuf, process::Command};

fn run_cli(
    root: &str,
    vars: Option<HashMap<String, String>>,
) -> (Command, String, String, String, String) {
    let output = "./tests/yml_test_files/output";
    let entry = "simple_book";
    let schema = "book-schema.yml";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("-r").arg(&root);
    cmd.arg("-e").arg(entry);
    cmd.arg("-s").arg(schema);
    cmd.arg("-o").arg(output);
    vars.unwrap_or_default()
        .into_iter()
        .for_each(|(key, value)| {
            cmd.arg("-v").arg(format!("{}={}", key, value));
        });

    (
        cmd,
        root.to_string(),
        output.to_string(),
        entry.to_string(),
        schema.to_string(),
    )
}

fn check_generated_file_ok(output: &str, file: &str, string_to_test: Option<&str>) {
    //read assembles
    let assembled_file_path = PathBuf::from(output).join(format!("{}.yml", file));
    let assembled_file = fs::read_to_string(assembled_file_path).unwrap();
    assert!(
        predicate::str::contains(string_to_test.unwrap_or("title: Juliette coupe le gateau"))
            .eval(&assembled_file)
    );
}

fn delete_output_folder(output: &str) {
    fs::remove_dir_all(PathBuf::from(output)).unwrap();
}

#[test]
#[serial]
fn it_should_run_cli() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/yml_test_files")
        .to_str()
        .unwrap()
        .to_string();

    let (mut cmd, root, output, file, schema) = run_cli(&root, None);

    let std_output = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Working in: {root}")))
        .stdout(predicate::str::contains(format!(
            "Assembling files: {file}"
        )))
        .stdout(predicate::str::contains(format!(
            "Validating from schema: {schema}"
        )))
        .stdout(predicate::str::contains("Assembling done!"))
        .stdout(predicate::str::contains(format!("Outputing in: {output}")))
        .get_output()
        .clone();

    println!("{}", String::from_utf8_lossy(&std_output.stdout));

    check_generated_file_ok(&output, &file, None);
    delete_output_folder(&output);
}

#[test]
#[serial]
fn it_should_run_cli_with_relative_root() {
    let (mut cmd, _, output, file, _) = run_cli("./tests/yml_test_files", None);

    let std_output = cmd.assert().success().get_output().clone();
    println!("{:?}", String::from_utf8_lossy(&std_output.stdout));

    check_generated_file_ok(&output, &file, None);
    delete_output_folder(&output);
}

#[test]
#[serial]
fn it_should_run_twice_and_completely_replace_file() {
    let (mut cmd, _, _, _, _) = run_cli("./tests/yml_test_files", None);

    let std_output = cmd.assert().success().get_output().clone();
    println!("{}", String::from_utf8_lossy(&std_output.stdout));

    let (mut cmd, _, output, file, _) = run_cli("./tests/yml_test_files", None);

    let std_output = cmd.assert().success().get_output().clone();
    println!("{}", String::from_utf8_lossy(&std_output.stdout));

    check_generated_file_ok(&output, &file, None);
    delete_output_folder(&output);
}

#[test]
#[serial]
fn it_should_fail_if_non_existing_root() {
    let (mut cmd, _, _, _, _) = run_cli("./non_existing_root", None);
    cmd.assert().failure();
}

#[test]
#[serial]
fn it_should_generate_file_with_ordered_entries() {
    let (mut cmd, _, output, _, _) = run_cli("./tests/yml_test_files", None);

    let std_output = cmd.assert().success().get_output().clone();
    println!("{}", String::from_utf8_lossy(&std_output.stdout));

    let assembled_file_path = PathBuf::from(&output).join("simple_book.yml");
    let assembled_file = fs::read_to_string(assembled_file_path).unwrap();
    predicate::str::contains("title: Juliette coupe le gateau\nsummary: L'anniversaire de Juliette tourne mal\nstory:\ncontent: Ca y est ! Elle a 21 ans, et a invité tout le monde à pré coustille. Malheureusement Juliette n'est pas très adroite et se coupe le doigt en coupant le gâteau. Elle est emmenée à l'hôpital et se fait recoudre le doigt. Elle est très déçue de rater sa fête d'anniversaire.\nchapter: 5").eval(&assembled_file);

    delete_output_folder(&output);
}

#[test]
#[serial]
fn it_should_input_base_variables() {
    let mut variables = HashMap::new();
    variables.insert("META".to_string(), "I'm a root variable".to_string());
    variables.insert("META2".to_string(), "I'm another variable".to_string());

    let (mut cmd, _, output, file, _) = run_cli("./tests/yml_test_files", Some(variables));

    let std_output = cmd.assert().success().get_output().clone();
    println!("{}", String::from_utf8_lossy(&std_output.stdout));

    check_generated_file_ok(&output, &file, Some("meta: I'm a root variable"));
    check_generated_file_ok(&output, &file, Some("meta2: I'm another variable"));
    delete_output_folder(&output);
}

#[test]
#[serial]
fn it_should_output_json_schema() {
    let (mut cmd, _, output, _, schema) = run_cli("./tests/yml_test_files", None);

    let std_output = cmd.assert().success().get_output().clone();
    println!("{}", String::from_utf8_lossy(&std_output.stdout));

    let schema_output = PathBuf::from(&output).join(schema).with_extension("json");
    let schema_output = fs::read_to_string(schema_output).unwrap();
    let schema_output = serde_json::from_str::<serde_json::Value>(&schema_output).unwrap();

    let color_is_string = schema_output
        .get("definitions")
        .unwrap()
        .get("cover")
        .unwrap()
        .get("properties")
        .unwrap()
        .get("color")
        .unwrap()
        .get("type")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("string");

    let size_is_integer = schema_output
        .get("definitions")
        .unwrap()
        .get("cover")
        .unwrap()
        .get("properties")
        .unwrap()
        .get("size")
        .unwrap()
        .get("type")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("integer");

    assert!(
        color_is_string && size_is_integer,
        "Color is not a string or size is not an integer",
    );

    delete_output_folder(&output);
}
