#![allow(clippy::missing_safety_doc)]
wit_bindgen::generate!();

use std::io::Read;
use std::path::Path;

use clap::builder::ValueParser;
use clap::{Arg, CommandFactory, FromArgMatches};
use dir_parser::push_dir;
use exports::wasi::cli::run::Guest as RunGuest;
use exports::wasmcloud::wash::subcommand::{Argument, Guest as SubcommandGuest, Metadata};
use wasi::cli::environment;
use wasi::filesystem::preopens::get_directories;
use wasi::filesystem::types::{Descriptor, DescriptorFlags, OpenFlags, PathFlags};
use wit2wadm::cli::Args;
use wit2wadm::{raw_component_to_wadm, raw_wit_to_wadm};

mod dir_parser;

impl From<&Arg> for Argument {
    fn from(arg: &Arg) -> Self {
        Self {
            description: arg.get_help().map(ToString::to_string).unwrap_or_default(),
            is_path: arg.get_value_parser().type_id() == ValueParser::path_buf().type_id(),
            required: arg.is_required_set(),
        }
    }
}

struct Wit2WadmPlugin;

// Our implementation of the wasi:cli/run interface
impl RunGuest for Wit2WadmPlugin {
    fn run() -> Result<(), ()> {
        let args = environment::get_arguments();

        let cmd = Args::command();
        let matches = match cmd.try_get_matches_from(args) {
            Ok(matches) => matches,
            Err(err) => {
                eprintln!("Error parsing arguments: {}", err);
                return Err(());
            }
        };
        let args = match Args::from_arg_matches(&matches) {
            Ok(args) => args,
            Err(err) => {
                eprintln!("Error parsing arguments: {}", err);
                return Err(());
            }
        };

        let name = args.app_name.unwrap_or_else(|| "wit2wadm".to_string());
        let description = args
            .app_description
            .unwrap_or_else(|| "A wasmCloud Application".to_string());
        let version = args.app_version.unwrap_or_else(|| "v0.1.0".to_string());
        let app_image = args
            .app_image
            .unwrap_or_else(|| format!("myregistry.io/{name}:{version}"));
        let manifest = match get_dir(&args.wit_folder_or_component) {
            // Ok mean this was a directory because we found a match. If we didn't, it was a file
            Ok(dir) => {
                let world_name = args.world_name.ok_or_else(|| {
                    eprintln!("world name is required when providing a WIT folder");
                })?;
                let wit = push_dir(dir, &args.wit_folder_or_component).map_err(|e| {
                    eprintln!("Error loading WIT from directory: {}", e);
                })?;
                raw_wit_to_wadm(wit, world_name, &name, description, &version, app_image)
                    .map_err(|e| eprintln!("Unable to convert wit to wadm manifest: {:?}", e))?
            }
            Err(_) => {
                let file = open_file(
                    args.wit_folder_or_component,
                    OpenFlags::empty(),
                    DescriptorFlags::READ,
                )
                .map_err(|e| {
                    eprintln!("Unable to open component file: {}", e);
                })?;
                let buf = read_file(file).map_err(|e| {
                    eprintln!("Unable to read component file: {}", e);
                })?;
                raw_component_to_wadm(buf, &name, description, &version, app_image)
                    .map_err(|e| eprintln!("Unable to convert component to wadm manifest: {}", e))?
            }
        };

        let yaml_result = serde_yaml::to_string(&manifest);
        match yaml_result {
            Ok(yaml_string) => {
                println!("{}", yaml_string);
                Ok(())
            }
            Err(err) => {
                eprintln!("Error serializing to YAML: {}", err);
                Err(())
            }
        }
    }
}

// Our plugin's metadata implemented for the subcommand interface
impl SubcommandGuest for Wit2WadmPlugin {
    fn register() -> Metadata {
        let cmd = Args::command();
        let (arguments, flags): (Vec<_>, Vec<_>) =
            cmd.get_arguments().partition(|arg| arg.is_positional());
        // There isn't a partition_map function without importing another crate
        let arguments = arguments
            .into_iter()
            .map(|arg| (arg.get_id().to_string(), Argument::from(arg)))
            .collect();
        let flags = flags
            .into_iter()
            .map(|arg| {
                (
                    arg.get_long()
                        .unwrap_or_else(|| arg.get_id().as_str())
                        .to_string(),
                    Argument::from(arg),
                )
            })
            .collect();
        Metadata {
            name: "Wit2Wadm Wash Plugin".to_string(),
            id: "wit2wadm".to_string(),
            description: cmd.get_about().map(|s| s.to_string()).unwrap_or_else(|| {
                "Generate a wadm manifest from a wit directory or component".to_string()
            }),
            author: "WasmCloud".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            flags,
            arguments,
        }
    }
}

fn get_dir(path: impl AsRef<Path>) -> Result<Descriptor, String> {
    get_directories()
        .into_iter()
        .find_map(|(dir, dir_path)| {
            (<std::string::String as std::convert::AsRef<Path>>::as_ref(&dir_path) == path.as_ref())
                .then_some(dir)
        })
        .ok_or_else(|| format!("Could not find directory {}", path.as_ref().display()))
}

/// Opens the given file. This should be the canonicalized path to the file.
pub(crate) fn open_file(
    path: impl AsRef<Path>,
    open_flags: OpenFlags,
    descriptor_flags: DescriptorFlags,
) -> Result<Descriptor, String> {
    let dir = path
        .as_ref()
        .parent()
        // I mean, if someone passed a path that is at the root, that probably wasn't a good idea
        .ok_or_else(|| {
            format!(
                "Could not find parent directory of {}",
                path.as_ref().display()
            )
        })?;
    let dir = get_dir(dir)?;
    dir.open_at(
        PathFlags::empty(),
        path.as_ref()
            .file_name()
            .ok_or_else(|| format!("Path did not have a file name: {}", path.as_ref().display()))?
            .to_str()
            .ok_or_else(|| "Path is not a valid string".to_string())?,
        open_flags,
        descriptor_flags,
    )
    .map_err(|e| format!("Failed to open file {}: {}", path.as_ref().display(), e))
}

fn read_file(dir: Descriptor) -> Result<Vec<u8>, String> {
    let mut body = dir
        .read_via_stream(0)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let mut buf = vec![];
    InputStreamReader::from(&mut body)
        .read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(buf)
}

// Helpers for reading from streams.
pub struct InputStreamReader<'a> {
    stream: &'a mut crate::wasi::io::streams::InputStream,
}

impl<'a> From<&'a mut crate::wasi::io::streams::InputStream> for InputStreamReader<'a> {
    fn from(stream: &'a mut crate::wasi::io::streams::InputStream) -> Self {
        Self { stream }
    }
}

impl std::io::Read for InputStreamReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use crate::wasi::io::streams::StreamError;
        use std::io;

        let n = buf
            .len()
            .try_into()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        match self.stream.blocking_read(n) {
            Ok(chunk) => {
                let n = chunk.len();
                if n > buf.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "more bytes read than requested",
                    ));
                }
                buf[..n].copy_from_slice(&chunk);
                Ok(n)
            }
            Err(StreamError::Closed) => Ok(0),
            Err(StreamError::LastOperationFailed(e)) => {
                Err(io::Error::new(io::ErrorKind::Other, e.to_debug_string()))
            }
        }
    }
}

export!(Wit2WadmPlugin);
