use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, read_dir},
    io,
    path::PathBuf,
};

use bpaf::{construct, long, positional, OptionParser, Parser};
use vel::VelInstance;

#[derive(Debug)]
struct Options {
    component_dirs: Option<Vec<PathBuf>>,
    file: PathBuf,
}

fn get_cli_options() -> OptionParser<Options> {
    let component_dirs = long("components")
        .short('c')
        .argument("COMPONENTS")
        .help("Path to a component or directory of components.")
        .many()
        .optional();
    let file = positional("FILE");

    construct!(Options {
        component_dirs,
        file
    })
    .to_options()
    .descr("A SSG tool inspired by Svelte.")
    .header("This tool is intended to be used in a build script.
    Inputs are provided as environment variables, which allows you to use the value of any environment variable inside your template files.
    Components are called by their filestem with the first letter capitalised automatically.
    Conflicts can occur if you have 2 files with the same filestem but different extension e.g. 'template.html' and 'template.css'.")
    .footer("Copyright of Name.
    Licensed under MPL-2.0, with source code available from 'https://git.garfunkles.space/velox'.
    If you end up depending on this project and or want to help support its development, you can make a donation at 'https://garfunkles.space/donate' <3")
    .version(env!("CARGO_PKG_VERSION"))
}

fn collect_files_recursively(dir: &PathBuf) -> Result<Vec<PathBuf>, io::Error> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                files.extend(collect_files_recursively(&path)?);
            } else {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn main() -> Result<(), Box<dyn Error>> {
    let passed_options = get_cli_options().run();

    let file_name = passed_options
        .file
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let file_contents = fs::read_to_string(&passed_options.file)?;

    let mut components = HashMap::from([(file_name.clone(), file_contents)]);

    if let Some(dirs) = passed_options.component_dirs {
        for dir in dirs {
            if dir.is_dir() {
                for file_path in collect_files_recursively(&dir)? {
                    let file_name = file_path.file_stem().unwrap().to_str().unwrap().to_string();
                    let file_contents = fs::read_to_string(&file_path).unwrap();
                    components.insert(file_name, file_contents);
                }
            }
        }
    }

    let inputs = HashMap::from_iter(env::vars());

    print!(
        "{}",
        VelInstance::new(components)
            .render(file_name, inputs, |element| { Some(element) })?
            .trim()
    );
    Ok(())
}
