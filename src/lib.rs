// lib.rs
#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!();

// use crate::exports::wasmcloud::tools::convert::KnownInterface;

#[cfg(target_arch = "wasm32")]
struct Wit2WadmComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::tools::convert::Guest for Wit2WadmComponent {
    fn component_to_wadm(component: Vec<u8>) -> Result<String, String> {
        let (resolve, _world) = match wit_component::decode(&component) {
            Ok(wit_component::DecodedWasm::Component(resolve, world)) => (resolve, world),
            Err(_) | Ok(wit_component::DecodedWasm::WitPackage(..)) => {
                return Err("Expected a WIT component".to_string());
            }
        };

        // TODO: accept world ID
        let manifest = wit2wadm::convert::wit2wadm(resolve, "root")
            .expect("should be able to convert to manifest");
        let yaml_result = serde_yaml::to_string(&manifest);
        match yaml_result {
            Ok(yaml_string) => Ok(yaml_string),
            Err(err) => Err(format!("Error serializing to YAML: {err}")),
        }
    }
}

#[cfg(target_arch = "wasm32")]
export!(Wit2WadmComponent);
