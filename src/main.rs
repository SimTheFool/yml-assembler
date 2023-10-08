use clap::Parser;
use std::{collections::HashMap, error::Error, path::PathBuf, sync::Arc, thread::JoinHandle};
use yml_assembler::{
    adapters::{AssemblyOutputFormat, PartReaderPort},
    lib_infras::{
        assembly_fs_output::AssemblyFSOutput, assembly_part_fs_reader::PartFSReader,
        schema_fs_output::SchemaFSOutput, schema_fs_reader::SchemaFSReader,
    },
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The directory your pyml files reside in
    #[arg(short, long)]
    root: PathBuf,

    /// The path to the pyml file to assemble (relative to root)
    #[arg(short, long)]
    entry: String,

    /// The path to the schema file to validate against (relative to root)
    #[arg(short, long)]
    schema: Option<String>,

    /// The path to the output folder
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// The format of the output file (json or yml)
    #[clap(long, short = 'f', default_value = "yml", value_enum)]
    format: AssemblyOutputFormat,

    /// Variables to insert in the pyml assembly
    #[arg(short, long, value_parser = parse_key_val::<String, String>)]
    vars: Option<Vec<(String, String)>>,
}

static DEFAULT_OUTPUT: &str = "output";

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn cli() -> Result<(), anyhow::Error> {
    let Cli {
        output,
        entry,
        root,
        schema,
        vars,
        format,
    } = Cli::parse();

    let display_variables = format!(
        "Using variables:{}",
        &vars
            .clone()
            .unwrap_or(vec![])
            .iter()
            .fold("".to_string(), |acc, (k, v)| format!(
                "{}\n{}={}",
                acc, k, v
            ))
    );
    let variables: HashMap<String, String> =
        HashMap::from_iter(vars.unwrap_or_default().into_iter());

    let outdir = PathBuf::from(DEFAULT_OUTPUT);
    let outdir = output.unwrap_or(outdir);

    println!("{}", display_variables);
    println!("Using format: {:?}", format);
    println!("Working in: {}", root.display());
    if let Some(schema) = schema.as_deref() {
        println!("Validating from schema: {}", schema);
    }
    println!("Outputing in: {}", outdir.display());

    let part_fs_reader = PartFSReader::new(root.clone());
    let entries = part_fs_reader.get_filepathes_from_glob(&entry)?;
    println!("Assembling files: {}", entries.clone().join(" "));

    let schema_fs_reader = SchemaFSReader::new(root.clone());
    let assembly_fs_output = AssemblyFSOutput::new(outdir.clone());
    let schema_fs_output = SchemaFSOutput::new(outdir.clone());

    let app = yml_assembler::App::new(
        Arc::new(part_fs_reader),
        Arc::new(schema_fs_reader),
        Arc::new(assembly_fs_output),
        Arc::new(schema_fs_output),
    );

    let wait_for_assemble = entries
        .iter()
        .map(|entry| {
            let entry = entry.clone();
            let app = app.clone();
            let schema = schema.clone();
            let variables = variables.clone();
            let format = format.clone();

            std::thread::spawn(move || {
                app.clone().compile_and_validate_yml(
                    &entry,
                    schema.as_deref(),
                    Some(variables.clone()),
                    &format,
                )
            })
        })
        .collect::<Vec<JoinHandle<_>>>();

    let join_handle_result: Result<(), anyhow::Error> = wait_for_assemble
        .into_iter()
        .map(|handle| handle.join())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!(format!("Could not join thread: {:?}", e)))?
        .iter()
        .try_fold((), |acc, x| {
            if let Err(e) = x {
                Err(anyhow::anyhow!(format!("Could not assemble: {:?}", e)))?;
            }
            Ok(acc)
        });

    join_handle_result?;

    Ok(())
}

fn main() {
    match cli() {
        Ok(_) => println!("Assembling done!"),
        Err(e) => {
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
