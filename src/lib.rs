// lib.rs
wit_bindgen::generate!();

// use crate::exports::wasmcloud::tools::convert::KnownInterface;

struct Wit2WadmComponent;

impl exports::wasmcloud::tools::convert::Guest for Wit2WadmComponent {
    fn component_to_wadm(component: Vec<u8>) -> Result<String, String> {
        let (resolve, _world) = match wit_component::decode(&component) {
            Ok(wit_component::DecodedWasm::Component(resolve, world)) => (resolve, world),
            Err(_) | Ok(wit_component::DecodedWasm::WitPackage(..)) => {
                return Err("Expected a WIT component".to_string());
            }
        };

        // TODO: accept world ID
        let manifest = wit2wadm_lib::convert::wit2wadm(resolve, "root")
            .expect("should be able to convert to manifest");
        let yaml_result = serde_yaml::to_string(&manifest);
        match yaml_result {
            Ok(yaml_string) => Ok(yaml_string),
            Err(err) => Err(format!("Error serializing to YAML: {err}")),
        }
    }
}

export!(Wit2WadmComponent);
