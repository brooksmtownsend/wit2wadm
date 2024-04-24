use std::path::Path;

use anyhow::{bail, Context as _};
use wadm::model::Manifest;
use wit_parser::{Resolve, World};

use crate::manifest;

pub fn wit2wadm_from_folder(
    wit_folder: impl AsRef<Path>,
    world_name: impl AsRef<str>,
) -> anyhow::Result<Manifest> {
    let mut resolve = Resolve::new();

    resolve
        .push_path(wit_folder)
        .context("should be able to load wits")?;

    let world = resolve
        .worlds
        .iter()
        .find_map(|(_, w)| (w.name == world_name.as_ref()).then_some(w))
        .cloned()
        .context("component world missing")
        .expect("should be able to find component world");

    let manifest = wit2wadm(resolve, &world).context("should be able to convert to manifest")?;

    Ok(manifest)
}

/// Loads a WIT component from a file and converts it to a wadm application manifest
pub fn wit2wadm_from_component(wit_component: impl AsRef<Path>) -> anyhow::Result<Manifest> {
    let wasm = std::fs::read(&wit_component).context("failed to read WIT component")?;
    let (resolve, world) =
        match wit_component::decode(&wasm).context("failed to decode WIT component")? {
            wit_component::DecodedWasm::Component(resolve, world) => (resolve, world),
            wit_component::DecodedWasm::WitPackage(..) => {
                bail!("binary-encoded WIT packages not currently supported")
            }
        };

    let world = resolve
        .worlds
        .iter()
        .find_map(|(id, w)| (id == world).then_some(w))
        .cloned()
        .context("component world missing")
        .expect("should be able to find component world");

    let manifest = wit2wadm(resolve, &world).context("should be able to convert to manifest")?;

    Ok(manifest)
}

/// Converts a component [Resolve] and [World] into a wadm application manifest
pub fn wit2wadm(resolve: Resolve, world: &World) -> anyhow::Result<Manifest> {
    let wit_parser::World {
        exports, imports, ..
    } = world;

    let manifest = manifest::create_manifest(
        // TODO: un-hardcode these values
        "appname",
        "appdesc",
        "appimage",
        "v0.0.1",
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
    // let yaml_result = serde_yaml::to_string(&manifest);
    // match yaml_result {
    //     Ok(yaml_string) => println!("{}", yaml_string),
    //     Err(err) => eprintln!("Error serializing to YAML: {}", err),
    // }

    Ok(manifest)
}
