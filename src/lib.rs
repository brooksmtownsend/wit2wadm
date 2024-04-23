// lib.rs
#[cfg(target_arch = "wasm32")]

wit_bindgen::generate!();

#[cfg(target_arch = "wasm32")]
struct Wit2WadmComponent;

#[cfg(target_arch = "wasm32")]
impl exports::wasmcloud::tools::convert::Guest for Wit2WadmComponent {
    fn component_to_wadm(component: Vec<u8>) -> Result<String, String> {
        Ok("ok".to_string())
    }
}

#[cfg(target_arch = "wasm32")]
export!(Wit2WadmComponent);
