use std::{
    collections::{HashMap, VecDeque},
    env,
    error::Error,
    fs::{self, read_dir},
    io,
    mem::discriminant,
    path::{Components, PathBuf},
};

use bpaf::{construct, long, positional, OptionParser, Parser};
use vel::VelInstance;

#[derive(Debug)]
struct Options {
    components: Vec<PathBuf>,
    file: PathBuf,
}

fn get_cli_options() -> OptionParser<Options> {
    let components = long("components")
        .short('c')
        .argument("COMPONENTS")
        .help("Path to a component or directory of components.")
        .many();
    let file = positional("FILE");

    construct!(Options {
        components,
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

fn collect_components_recursively(mut dirs: VecDeque<PathBuf>) -> Vec<(String, String)> {
    let mut components = Vec::new();

    while let Some(dir) = dirs.pop_front() {
        if let Ok(entries) = dir.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push_back(path);
                } else if let (Some(file_name), Ok(file_contents)) = (
                    path.file_stem().and_then(|s| s.to_str().map(String::from)),
                    fs::read_to_string(&path),
                ) {
                    components.push((file_name, file_contents));
                }
            }
        }
    }

    components
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

    components.extend(
        collect_components_recursively(
            passed_options
                .components
                .into_iter()
                .filter_map(|path| path.canonicalize().ok())
                .collect(),
        )
        .into_iter(),
    );

    let inputs = HashMap::from_iter(env::vars());

    print!(
        "{}",
        VelInstance::new(components)
            .render(file_name, inputs, |element| { Some(element) })?
            .trim()
    );
    Ok(())
}
