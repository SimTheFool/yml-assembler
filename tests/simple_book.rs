use serde_yaml;
use yml_assembler::adapters::AssemblyOutputFormat;

pub mod test_infra;

#[derive(Debug, serde::Deserialize)]
struct BookFromYml {
    title: String,
    summary: String,
    story: StoryFromYml,
    covers: Vec<CoverFromYml>,
    tags: Vec<String>,
    page: PagesFromYml,
}

#[derive(Debug, serde::Deserialize)]
struct PagesFromYml {
    number: f64,
    weight: f64,
}

#[derive(Debug, serde::Deserialize)]
struct StoryFromYml {
    content: String,
    chapter: f64,
}

#[derive(Debug, serde::Deserialize)]
struct CoverFromYml {
    color: String,
    size: f64,
}

static TEST_FILE: &str = "simple_book";

#[tokio::test]
async fn it_should_aggregate_filesystem_yml_files() {
    let (app, assembly_output, _) = test_infra::get_test_app();
    app.compile_and_validate_yml(TEST_FILE, None, None, &AssemblyOutputFormat::Yml)
        .unwrap();
    let yml = assembly_output
        .get_yml_output()
        .unwrap()
        .get(TEST_FILE)
        .unwrap()
        .clone();

    let book: BookFromYml = serde_yaml::from_value(yml).unwrap();

    assert_eq!(book.title, "Juliette coupe le gateau");
    assert_eq!(book.summary, "L'anniversaire de Juliette tourne mal");
    assert_eq!(book.story.content, "Ca y est ! Elle a 21 ans, et a invité tout le monde à pré coustille. Malheureusement Juliette n'est pas très adroite et se coupe le doigt en coupant le gâteau. Elle est emmenée à l'hôpital et se fait recoudre le doigt. Elle est très déçue de rater sa fête d'anniversaire.");
    assert_eq!(book.story.chapter, 5.0);
}

#[tokio::test]
async fn it_should_mix_properties() {
    let (app, assembly_output, _) = test_infra::get_test_app();
    app.compile_and_validate_yml(TEST_FILE, None, None, &AssemblyOutputFormat::Yml)
        .unwrap();
    let yml = assembly_output
        .get_yml_output()
        .unwrap()
        .get(TEST_FILE)
        .unwrap()
        .clone();

    let book: BookFromYml = serde_yaml::from_value(yml).unwrap();

    assert_eq!(book.covers.len(), 4);
    assert_eq!(book.tags.len(), 3);

    let yellow_cover = book.covers.iter().find(|c| c.color == "yellow");
    assert_eq!(yellow_cover.unwrap().size, 36 as f64);

    let rose_cover = book.covers.iter().find(|c| c.color == "rose");
    assert_eq!(rose_cover.unwrap().size, 15 as f64);

    let red_cover = book.covers.iter().find(|c| c.color == "red");
    assert_eq!(red_cover.unwrap().size, 10 as f64);

    let black_cover = book.covers.iter().find(|c| c.color == "black");
    assert_eq!(black_cover.unwrap().size, 20 as f64);

    let investigation_tag = book.tags.iter().find(|t| t == &"ivestigation");
    assert!(investigation_tag.is_some());

    let adult_tag = book.tags.iter().find(|t| t == &"adult");
    assert!(adult_tag.is_some());

    let horror_tag = book.tags.iter().find(|t| t == &"horror");
    assert!(horror_tag.is_some());
}

#[tokio::test]
async fn it_should_validate_from_json() {
    let (app, _, _) = test_infra::get_test_app();
    app.compile_and_validate_yml(
        TEST_FILE,
        Some("book-schema.json"),
        None,
        &AssemblyOutputFormat::Yml,
    )
    .unwrap();
}

#[tokio::test]
async fn it_should_validate_from_yml() {
    let (app, _, _) = test_infra::get_test_app();
    app.compile_and_validate_yml(
        TEST_FILE,
        Some("book-schema.yml"),
        None,
        &AssemblyOutputFormat::Yml,
    )
    .unwrap();
}

#[tokio::test]
async fn it_should_transform_properties() {
    let (app, assembly_output, _) = test_infra::get_test_app();
    app.compile_and_validate_yml(TEST_FILE, None, None, &AssemblyOutputFormat::Yml)
        .unwrap();
    let yml = assembly_output
        .get_yml_output()
        .unwrap()
        .get(TEST_FILE)
        .unwrap()
        .clone();

    let book: BookFromYml = serde_yaml::from_value(yml).unwrap();

    assert_eq!(book.page.number, 40 as f64);
    assert_eq!(book.page.weight, 10.0);
}
