use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author = "wasmCloud", arg_required_else_help = true)]
/// wit2wadm is a tool for converting a WIT directory or a WebAssembly component into a deployable https://github.com/wasmcloud/wadm manifests.
/// Either supply a WIT folder and a world name or a WebAssembly component and the resulting manifest will be printed to stdout.
/// See https://github.com/brooksmtownsend/wit2wadm for more information.
pub struct Args {
    #[clap(name = "wit_folder_or_component")]
    /// The path to a WIT folder including dependencies and a world, or a built Wasm component
    pub wit_folder_or_component: PathBuf,
    /// The world name to use to convert to a manifest, required if a WIT folder is provided
    #[clap(name = "world_name")]
    pub world_name: Option<String>,
    /// The name of the application to use in the manifest
    #[clap(long = "name")]
    pub app_name: Option<String>,
    /// The description of the application to use in the manifest
    #[clap(long = "description")]
    pub app_description: Option<String>,
    /// The version of the application to use in the manifest
    #[clap(long = "app-version")]
    pub app_version: Option<String>,
    /// The image to use in the manifest
    #[clap(long = "image")]
    pub app_image: Option<String>,
}
