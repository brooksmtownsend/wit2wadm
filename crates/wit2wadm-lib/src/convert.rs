use anyhow::Context as _;
use wit_parser::Resolve;

use crate::manifest;

/// Converts a component [Resolve] and world name into a wadm application manifest
pub fn wit2wadm(resolve: Resolve, world_name: &str) {
    let wit_parser::World {
        exports, imports, ..
    } = resolve
        .worlds
        .iter()
        .find_map(|(_, w)| (w.name == world_name).then_some(w))
        .context("component world missing")
        .expect("should be able to find component world");

    let manifest = manifest::create_manifest(
        // TODO: un-hardcode these values
        "appname",
        "appdesc",
        "appversion",
        "appimage",
        imports
            .iter()
            .map(|(id, _)| resolve.name_world_key(id))
            .collect(),
        exports
            .iter()
            .map(|(id, _)| resolve.name_world_key(id))
            .collect(),
    );

    // Print the manifest as YAML
    let yaml_result = serde_yaml::to_string(&manifest);
    match yaml_result {
        Ok(yaml_string) => println!("{}", yaml_string),
        Err(err) => eprintln!("Error serializing to YAML: {}", err),
    }
}
