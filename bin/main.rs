use std::path::PathBuf;

use anyhow::Context as _;
use clap::Parser;
use wit2wadm::convert::{wit2wadm_from_component, wit2wadm_from_folder};

#[derive(Parser)]
#[clap(version, author = "wasmCloud", arg_required_else_help = true)]
/// wit2wasm is a tool for converting a WIT directory or a WebAssembly component into a deployable https://github.com/wasmcloud/wadm manifests.
/// Either supply a WIT folder and a world name or a WebAssembly component and the resulting manifest will be printed to stdout.
/// See https://github.com/brooksmtownsend/wit2wadm for more information.
struct Args {
    #[clap(name = "wit_folder_or_component")]
    /// The path to a WIT folder including dependencies and a world, or a built Wasm component
    wit_folder_or_component: PathBuf,
    /// The world name to use to convert to a manifest, required if a WIT folder is provided
    #[clap(name = "world_name")]
    world_name: Option<String>,
}

fn main() {
    let args = Args::parse();
    let manifest = if args.wit_folder_or_component.is_dir() {
        let world_name = args
            .world_name
            .expect("world name is required when providing a WIT folder");
        wit2wadm_from_folder(&args.wit_folder_or_component, &world_name)
    } else {
        wit2wadm_from_component(args.wit_folder_or_component)
    }
    .context("failed to convert WIT to WADM")
    .expect("should be able to convert WIT to WADM");

    let yaml_result = serde_yaml::to_string(&manifest);
    match yaml_result {
        Ok(yaml_string) => println!("{}", yaml_string),
        Err(err) => eprintln!("Error serializing to YAML: {}", err),
    }
}
