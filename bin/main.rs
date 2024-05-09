use anyhow::Context as _;
use clap::Parser;
use wit2wadm::{cli::Args, wit2wadm_from_component, wit2wadm_from_folder};

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
            world_name,
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
