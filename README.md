# wit2wadm

wit2wadm is a tool for converting a WIT directory or a WebAssembly component into a deployable [wadm](https://github.com/wasmcloud/wadm) manifest.

## Usage

### 🏗 wit2wadm Library

The majority of the logic for this conversion is included in the [wit2wadm crate](./crates/wit2wadm/). Primary usage comes from importing the `wit2wadm::component_to_wasm` function, which takes a `wit_parser::Resolve` and the name of a WIT world.

⚠️️ The API is extremely experimental, so expect breaking changes if you're using the library directly.

### 🏃 wit2wadm CLI

You can run the basic conversion from a WIT directory to a Wadm manifest by running the binary:

```bash
# Optionally, you can generate a new component if you don't have a WIT directory handy
# wash new component -t hello-world-rust hello
# cargo run -- <wit_directory> <world_name>
cargo run -- ./hello/wit hello
```

### 🐢 wit2wadm Component

You can build a component that is capable of interpreting a WebAssembly component and returning a Wadm manifest by running `wash build`.

🔮 In the future, this repository will include a Wadm manifest that will allow you to run `wit2wadm` as a wasmCloud application.

### 🌐 Web User Interface

Packaged in the [docs](./docs/) directory is a basic user interface that allows you to drag-and-drop a WebAssembly component and execute the `wit2wadm` component in the browser.

In order to run the UI, simply execute the script or the commands contained in [`./ui.sh`](./ui.sh). You will need [wash](https://wasmcloud.com/docs/installation), [jco](https://github.com/bytecodealliance/jco), and a Python3 installation to run the script.

You can also access this application at any time at [https://brooksmtownsend.github.io/wit2wadm/](https://brooksmtownsend.github.io/wit2wadm/)
