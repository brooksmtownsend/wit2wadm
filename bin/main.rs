use std::path::PathBuf;

use anyhow::Context as _;
use clap::Parser;
use wit2wadm::convert::{wit2wadm_from_component, wit2wadm_from_folder};

#[derive(Parser)]
#[clap(author = "wasmCloud", arg_required_else_help = true)]
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
    /// The name of the application to use in the manifest
    #[clap(long = "name")]
    app_name: Option<String>,
    /// The description of the application to use in the manifest
    #[clap(long = "description")]
    app_description: Option<String>,
    /// The version of the application to use in the manifest
    #[clap(long = "version")]
    app_version: Option<String>,
    /// The image to use in the manifest
    #[clap(long = "image")]
    app_image: Option<String>,
}

fn main() {
    let args = Args::parse();

    let name = args.app_name.unwrap_or_else(|| "wit2wadm".to_string());
    let description = args
        .app_description
        .unwrap_or_else(|| "A wasmCloud Application".to_string());
    let version = args.app_version.unwrap_or_else(|| "v0.1.0".to_string());

    let manifest = if args.wit_folder_or_component.is_dir() {
        let world_name = args
            .world_name
            .expect("world name is required when providing a WIT folder");
        wit2wadm_from_folder(
            &args.wit_folder_or_component,
            &world_name,
            &name,
            description,
            &version,
            args.app_image
                .unwrap_or_else(|| format!("myregistry.io/{name}:{version}")),
        )
    } else {
        wit2wadm_from_component(
            &args.wit_folder_or_component,
            name,
            description,
            version,
            args.app_image.unwrap_or_else(|| {
                format!(
                    "file://./{}",
                    args.wit_folder_or_component.to_string_lossy()
                )
            }),
        )
    }
    .context("failed to convert WIT to WADM")
    .expect("should be able to convert WIT to WADM");

    let yaml_result = serde_yaml::to_string(&manifest);
    match yaml_result {
        Ok(yaml_string) => println!("{}", yaml_string),
        Err(err) => eprintln!("Error serializing to YAML: {}", err),
    }
}
