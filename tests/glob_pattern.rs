use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use predicates::prelude::predicate;
use predicates::Predicate;
use serial_test::serial;
use std::{fs, path::PathBuf, process::Command};

pub mod test_infra;

#[tokio::test]
#[serial]
async fn it_should_assemble_glob_pattern_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/yml_test_files")
        .to_str()
        .unwrap()
        .to_string();
    let output = "./tests/yml_test_files/glob_entry_output";
    let entry = "glob/entry_*";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("-r").arg(&root);
    cmd.arg("-e").arg(entry);
    cmd.arg("-o").arg(output);

    let std_output = cmd.assert().success().get_output().clone();

    println!("{}", String::from_utf8_lossy(&std_output.stdout));

    let entry_a_path = PathBuf::from(output).join(format!("glob/entry_a.yml"));
    let entry_a_file = fs::read_to_string(entry_a_path).unwrap();
    assert!(predicate::str::contains("- a").eval(&entry_a_file));
    assert!(predicate::str::contains("- b").eval(&entry_a_file));

    let entry_b_path = PathBuf::from(output).join(format!("glob/entry_b.yml"));
    let entry_b_file = fs::read_to_string(entry_b_path).unwrap();
    assert!(predicate::str::contains("a: test A").eval(&entry_b_file));
    assert!(predicate::str::contains("b: test B").eval(&entry_b_file));

    let phantom_path = PathBuf::from(output).join(format!("glob/phantom.yml"));
    let phantom_file = fs::read_to_string(phantom_path);
    assert!(phantom_file.is_err());

    fs::remove_dir_all(PathBuf::from(output)).unwrap();
}

#[tokio::test]
#[serial]
async fn it_should_assemble_glob_pattern_folders() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/yml_test_files")
        .to_str()
        .unwrap()
        .to_string();
    let output = "./tests/yml_test_files/glob_entry_output";
    let entry = "_*/index";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("-r").arg(&root);
    cmd.arg("-e").arg(entry);
    cmd.arg("-o").arg(output);

    let std_output = cmd.assert().success().get_output().clone();

    println!("{}", String::from_utf8_lossy(&std_output.stdout));

    let entry_a_path = PathBuf::from(output).join(format!("_glob_a/index.yml"));
    let entry_a_file = fs::read_to_string(entry_a_path).unwrap();
    assert!(predicate::str::contains("a: 0").eval(&entry_a_file));
    assert!(predicate::str::contains("b: 1").eval(&entry_a_file));

    let entry_b_path = PathBuf::from(output).join(format!("_glob_b/index.yml"));
    let entry_b_file = fs::read_to_string(entry_b_path).unwrap();
    assert!(predicate::str::contains("- a").eval(&entry_b_file));
    assert!(predicate::str::contains("- b").eval(&entry_b_file));
    
    fs::remove_dir_all(PathBuf::from(output)).unwrap();
}
