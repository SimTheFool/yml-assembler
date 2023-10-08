use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use predicates::prelude::predicate;
use predicates::Predicate;
use serial_test::serial;
use std::{fs, path::PathBuf, process::Command};

pub mod test_infra;

#[tokio::test]
#[serial]
async fn it_should_load_from_cache() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/yml_test_files")
        .to_str()
        .unwrap()
        .to_string();
    let output = "./tests/yml_test_files/cache_test_output";
    let entry = "load_from_cache";

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("-r").arg(&root);
    cmd.arg("-e").arg(entry);
    cmd.arg("-o").arg(output);

    let std_output = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "reading: stories/car_crash_bis"
        )))
        .stdout(predicate::str::contains(format!(
            "reading from cache: stories/car_crash_bis"
        )))
        .get_output()
        .clone();

    println!("{}", String::from_utf8_lossy(&std_output.stdout));

    let assembled_file_path = PathBuf::from(output).join(format!("{}.yml", entry));
    let assembled_file = fs::read_to_string(assembled_file_path).unwrap();
    assert!(predicate::str::contains("It's a AUDI car crash").eval(&assembled_file));
    assert!(predicate::str::contains("It's a BMW car crash").eval(&assembled_file));
    fs::remove_dir_all(PathBuf::from(output)).unwrap();
}
