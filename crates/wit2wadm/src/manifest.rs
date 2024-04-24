use std::collections::BTreeMap;

use wadm::model::{
    Component, ComponentProperties, Manifest, Metadata, Properties, Specification,
    SpreadScalerProperty, Trait, TraitProperty, APPLICATION_KIND, LINK_TRAIT, OAM_VERSION,
    SPREADSCALER_TRAIT,
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

fn manifest_for_component(
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

    let exports_for_manifest = combine_interfaces(
        exports
            .iter()
            .filter_map(|export| {
                DirectionalInterface::parse_for_manifest(export, interface::Direction::Export)
            })
            .collect::<Vec<_>>(),
    );

    let export_components = exports_for_manifest.iter().filter_map(|export| {
        // When a component has an export, it's a different provider component in the manifest that will link to the component
        // so we can simply generate the link and then create a new component with that link
        export.to_source_link_property(name).and_then(|link| {
            export.to_capability_component().map(|component| Component {
                traits: Some(vec![Trait {
                    trait_type: LINK_TRAIT.to_string(),
                    properties: TraitProperty::Link(link),
                }]),
                ..component
            })
        })
    });

    // Ensure the component has a spreadscaler trait
    let mut traits = vec![Trait {
        trait_type: SPREADSCALER_TRAIT.to_string(),
        properties: TraitProperty::SpreadScaler(SpreadScalerProperty {
            instances: 1,
            spread: vec![],
        }),
    }];
    traits.extend(link_properties);
    component.traits = Some(traits);

    let mut out_vec = Vec::new();
    out_vec.push(component);
    out_vec.extend(provider_components.chain(export_components));

    out_vec
}

#[cfg(test)]
mod test {
    #[test]
    fn test_manifest_for_component() {}
}
