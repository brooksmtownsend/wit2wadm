#[cfg(feature = "cli")]
pub mod cli;
mod interface;
mod manifest;

use std::path::Path;

use anyhow::{bail, Context as _};
use wadm::model::Manifest;
use wit_parser::{Resolve, UnresolvedPackage, World};

/// Loads WIT world and deps from a folder and converts it to a wadm application manifest
pub fn wit2wadm_from_folder(
    wit_folder: impl AsRef<Path>,
    world_name: impl AsRef<str>,
    name: impl AsRef<str>,
    description: impl AsRef<str>,
    version: impl AsRef<str>,
    image: impl AsRef<str>,
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

    let manifest = wit2wadm(resolve, &world, name, description, version, image)
        .context("should be able to convert to manifest")?;

    Ok(manifest)
}

/// Takes a vec of UnresolvedPackages (from wit) and converts it to a wadm application manifest. The
/// vec of unresolved packages should be ordered properly. This function is mostly for use by a
/// component
pub fn raw_wit_to_wadm(
    packages: Vec<UnresolvedPackage>,
    world_name: impl AsRef<str>,
    name: impl AsRef<str>,
    description: impl AsRef<str>,
    version: impl AsRef<str>,
    image: impl AsRef<str>,
) -> anyhow::Result<Manifest> {
    let mut resolve = Resolve::new();

    for package in packages {
        resolve
            .push(package)
            .context("failed to load wit document")?;
    }

    let world = resolve
        .worlds
        .iter()
        .find_map(|(_, w)| (w.name == world_name.as_ref()).then_some(w))
        .cloned()
        .context("component world missing")
        .expect("should be able to find component world");

    wit2wadm(resolve, &world, name, description, version, image)
}

/// Loads a WIT component from a file and converts it to a wadm application manifest
pub fn wit2wadm_from_component(
    wit_component: impl AsRef<Path>,
    name: impl AsRef<str>,
    description: impl AsRef<str>,
    version: impl AsRef<str>,
    image: impl AsRef<str>,
) -> anyhow::Result<Manifest> {
    let wasm = std::fs::read(&wit_component).context("failed to read WIT component")?;
    raw_component_to_wadm(wasm, name, description, version, image)
        .context("should be able to convert to manifest")
}

/// Converts a raw WIT component to a wadm application manifest. Call this with the bytes of the
/// component loaded from a file or other source
pub fn raw_component_to_wadm(
    component: Vec<u8>,
    name: impl AsRef<str>,
    description: impl AsRef<str>,
    version: impl AsRef<str>,
    image: impl AsRef<str>,
) -> anyhow::Result<Manifest> {
    let (resolve, world) =
        match wit_component::decode(&component).context("failed to decode WIT component")? {
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

    wit2wadm(resolve, &world, name, description, version, image)
}

/// Converts a component [Resolve] and [World] into a wadm application manifest
pub fn wit2wadm(
    resolve: Resolve,
    world: &World,
    name: impl AsRef<str>,
    description: impl AsRef<str>,
    version: impl AsRef<str>,
    image: impl AsRef<str>,
) -> anyhow::Result<Manifest> {
    let wit_parser::World {
        exports, imports, ..
    } = world;

    let manifest = manifest::create_manifest(
        name.as_ref(),
        description.as_ref(),
        version.as_ref(),
        image.as_ref(),
        imports
            .iter()
            .map(|(id, _)| resolve.name_world_key(id))
            .collect(),
        exports
            .iter()
            .map(|(id, _)| resolve.name_world_key(id))
            .collect(),
    );

    Ok(manifest)
}
