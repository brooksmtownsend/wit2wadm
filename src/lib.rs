// lib.rs
#[cfg(target_arch = "wasm32")]
use anyhow::Context as _;

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!();

#[cfg(target_arch = "wasm32")]
struct Wit2WadmComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::tools::convert::Guest for Wit2WadmComponent {
    fn component_to_wadm(
        component: Vec<u8>,
        name: String,
        description: String,
        version: String,
        image: String,
    ) -> Result<String, String> {
        let (resolve, world) = match wit_component::decode(&component) {
            Ok(wit_component::DecodedWasm::Component(resolve, world)) => (resolve, world),
            Err(_) | Ok(wit_component::DecodedWasm::WitPackage(..)) => {
                return Err("Expected a WIT component".to_string());
            }
        };

        let world = resolve
            .worlds
            .iter()
            .find_map(|(id, w)| (id == world).then_some(w))
            .cloned()
            .context("component world missing")
            .expect("should be able to find component world");

        let (name, description, version, image) =
            resolve_empty_strings(name, description, version, image);
        let manifest =
            wit2wadm::convert::wit2wadm(resolve, &world, &name, &description, &version, &image)
                .expect("should be able to convert to manifest");

        let yaml_result = serde_yaml::to_string(&manifest);
        match yaml_result {
            Ok(yaml_string) => Ok(yaml_string),
            Err(err) => Err(format!("Error serializing to YAML: {err}")),
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn resolve_empty_strings(
    name: String,
    description: String,
    version: String,
    image: String,
) -> (String, String, String, String) {
    let name = if name.is_empty() {
        "APPLICATION NAME".to_string()
    } else {
        name
    };
    let description = if description.is_empty() {
        "APPLICATION DESCRIPTION".to_string()
    } else {
        description
    };
    let version = if version.is_empty() {
        "v0.0.1".to_string()
    } else {
        version
    };
    let image = if image.is_empty() {
        "APPLICATION IMAGE".to_string()
    } else {
        image
    };
    (name, description, version, image)
}

#[cfg(target_arch = "wasm32")]
export!(Wit2WadmComponent);
