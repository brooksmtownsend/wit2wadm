use std::collections::BTreeMap;

use wadm::model::{
    Component, ComponentProperties, Manifest, Metadata, Properties, Specification, Trait,
    TraitProperty, APPLICATION_KIND, LINK_TRAIT, OAM_VERSION,
};

use crate::interface::{self, combine_interfaces, DirectionalInterface};

/// Create a manifest from metadata about a component and a list of imports and exports
/// specified in that component's WIT world
pub fn create_manifest(
    name: &str,
    description: &str,
    version: &str,
    image: &str,
    imports: Vec<String>,
    exports: Vec<String>,
) -> Manifest {
    let components = manifest_for_component(name, image, imports, exports);
    Manifest {
        spec: Specification { components },
        ..base_manifest(name, description, version)
    }
}

/// Scaffolds the base manifest for an application
///
/// ```rust
/// let manifest = base_manifest("echo", "An Echo Application", "v0.1.0");
///
/// assert_eq!(manifest, Manifest {
///   api_version: wadm::model::OAM_VERSION.to_string();
///   kind: wadm::model::APPLICATION_KIND.to_string(),
///   metadata: Metadata {
///     name: "echo".to_string(),
///     annotations: BTreeMap::from_iter([
///       ("version".to_string(), "v0.1.0".to_string()),
///       ("description".to_string(), "An Echo Application".to_string()),
///     ]),
///     labels: BTreeMap::from_iter([("generated-by".to_string(), "wit2wadm".to_string())]),
///   },
///   spec: Specification { components: vec![] },
/// }
/// });
pub fn base_manifest(name: &str, description: &str, version: &str) -> Manifest {
    // Define metadata for the manifest
    let metadata = Metadata {
        name: name.to_string(),
        annotations: BTreeMap::from_iter([
            ("version".to_string(), version.to_string()),
            ("description".to_string(), description.to_string()),
        ]),
        labels: BTreeMap::from_iter([("generated-by".to_string(), "wit2wadm".to_string())]),
    };

    // Create the manifest
    Manifest {
        api_version: OAM_VERSION.to_string(),
        kind: APPLICATION_KIND.to_string(),
        metadata,
        spec: Specification { components: vec![] },
    }
}

fn manifest_for_component<'a>(
    name: &str,
    image: &str,
    imports: Vec<String>,
    exports: Vec<String>,
) -> Vec<Component> {
    let mut component = Component {
        name: name.to_string(),
        properties: Properties::Component {
            properties: ComponentProperties {
                image: image.to_string(),
                id: None,
                config: Vec::new(),
            },
        },
        traits: None,
    };

    let imports_for_manifest = combine_interfaces(
        imports
            .iter()
            .filter_map(|import| {
                DirectionalInterface::parse_for_manifest(import, interface::Direction::Import)
            })
            .collect::<Vec<_>>(),
    );

    let link_properties: Vec<Trait> = imports_for_manifest
        .iter()
        .filter_map(|import| import.to_target_link_property())
        .map(|properties| Trait {
            trait_type: LINK_TRAIT.to_string(),
            properties: TraitProperty::Link(properties),
        })
        .collect();

    let provider_components = imports_for_manifest
        .iter()
        .filter_map(|import| import.to_capability_component());

    component.traits = Some(link_properties);

    let mut out_vec = Vec::new();
    out_vec.push(component);
    out_vec.extend(provider_components);

    out_vec
}

#[cfg(test)]
mod test {
    use crate::manifest::base_manifest;

    use super::manifest_for_component;

    #[test]
    fn wit2wadm() {
        let mut manifest = base_manifest("echo", "schmecho", "vecho");
        let basic_manifest = manifest_for_component(
            "echo",
            "myref.io",
            vec![
                "wasi:io/streams@0.2.0".to_string(),
                "wasmcloud:messaging/consumer@0.2.0".to_string(),
                "wasi:logging/logging".to_string(),
                "wasi:http/outgoing-handler@0.2.0".to_string(),
            ],
            vec![],
        );

        manifest.spec.components = basic_manifest;

        // Print the manifest as YAML
        let yaml_result = serde_yaml::to_string(&manifest);
        match yaml_result {
            Ok(yaml_string) => println!("{}", yaml_string),
            Err(err) => eprintln!("Error serializing to YAML: {}", err),
        }
    }
}
