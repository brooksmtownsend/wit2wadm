use std::collections::HashMap;

use wadm::model::{CapabilityProperties, Component, LinkProperty, Properties};

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
/// The direction of an interface, either import or export
pub enum Direction {
    Import,
    Export,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::Import => "target",
            Direction::Export => "source",
        }
    }
}

/// An individual directional interface parsed from a WIT [Resolve]
#[derive(Clone)]
pub struct DirectionalInterface<'a> {
    pub namespace: &'a str,
    pub package: &'a str,
    pub interface: &'a str,
    version: Option<&'a str>,
    pub direction: Direction,
}

/// Represents a combined interface
pub struct CombinedInterface<'a> {
    pub namespace: &'a str,
    pub package: &'a str,
    pub version: Option<&'a str>,
    pub direction: Direction,
    pub interfaces: Vec<&'a str>,
}

/// Combines a Vec of DirectionalInterface into a Vec of interfaces where
/// the namespace, package, version, and direction are the same but the
/// interfaces are combined into a Vec.
pub fn combine_interfaces(interfaces: Vec<DirectionalInterface>) -> Vec<CombinedInterface> {
    let mut interface_map: HashMap<(&str, &str, Option<&str>, Direction), Vec<&str>> =
        HashMap::new();

    for interface in interfaces {
        let key = (
            interface.namespace,
            interface.package,
            interface.version,
            interface.direction,
        );
        interface_map
            .entry(key)
            .or_default()
            .push(interface.interface);
    }

    interface_map
        .into_iter()
        .map(
            |((namespace, package, version, direction), interfaces)| CombinedInterface {
                namespace,
                package,
                version,
                direction,
                interfaces,
            },
        )
        .collect()
}

impl<'a> DirectionalInterface<'a> {
    /// Parse a [DirectionalInterface] from an import or export, returning
    /// `None` if the interface is invalid or not supported in a wadm manifest.
    ///
    /// For example, an interface "foo:bar" is invalid and would return `None`,
    /// and `wasi:io/error@0.2.0` is handled automatically by the host and doesn't
    /// need to be included in the manifest.
    //TODO: just parse regular, then check if it's in the list of supported interfaces?
    pub fn parse_for_manifest(interface: &'a str, direction: Direction) -> Option<Self> {
        match interface {
            "wasi:http/incoming-handler@0.2.0" => Some(DirectionalInterface {
                namespace: "wasi",
                package: "http",
                interface: "incoming-handler",
                version: Some("0.2.0"),
                direction,
            }),
            "wasi:http/outgoing-handler@0.2.0" => Some(DirectionalInterface {
                namespace: "wasi",
                package: "http",
                interface: "outgoing-handler",
                version: Some("0.2.0"),
                direction,
            }),
            "wasmcloud:messaging/consumer@0.2.0" => Some(DirectionalInterface {
                namespace: "wasmcloud",
                package: "messaging",
                interface: "consumer",
                version: Some("0.2.0"),
                direction,
            }),
            "wasmcloud:messaging/handler@0.2.0" => Some(DirectionalInterface {
                namespace: "wasmcloud",
                package: "messaging",
                interface: "handler",
                version: Some("0.2.0"),
                direction,
            }),
            // These interfaces are handled automatically in the host and do not need to be
            // included in the manifest
            "wasi:logging/logging"
            | "wasi:random/random@0.2.0"
            | "wasi:http/types@0.2.0"
            | "wasmcloud:messaging/types@0.2.0"
            | "wasi:blobstore/types@0.2.0-draft"
            | "wasi:config/runtime@0.2.0-draft"
            | "wasmcloud:bus/lattice@1.0.0" => None,
            s if s.starts_with("wasi:io/")
                || s.starts_with("wasi:clocks/")
                || s.starts_with("wasi:filesystem/")
                || s.starts_with("wasi:cli/")
                || s.starts_with("wasi:sockets/") =>
            {
                None
            }
            // For all other (custom) interfaces, parse them manually requiring at least
            // a namespace, package, and interface name
            s => {
                let (namespace, rest) = s
                    .split_once(':')
                    .expect("invalid interface: missing namespace");
                let (package, rest) = rest
                    .split_once('/')
                    .expect("invalid interface: missing package");
                let (interface, version) = rest
                    .split_once('@')
                    .map(|(i, v)| (i, Some(v)))
                    .unwrap_or((rest, None));
                Some(DirectionalInterface {
                    namespace,
                    package,
                    interface,
                    version,
                    direction,
                })
            }
        }
    }
}

impl<'a> CombinedInterface<'a> {
    pub fn name(&self) -> String {
        format!(
            "{}:{}-{}",
            self.namespace,
            self.package,
            self.direction.as_str()
        )
    }

    fn capability_image(&self) -> Option<String> {
        match (self.namespace, self.package, self.direction) {
            // These interfaces are handled automatically in the host and do not have
            // associated mages
            // WASI Standard Interfaces, mostly in 0.2
            ("wasi", "logging", _)
            | ("wasi", "io", _)
            | ("wasi", "clocks", _)
            | ("wasi", "sockets", _)
            | ("wasi", "filesystem", _)
            | ("wasi", "cli", _) => None,
            // Types or Specific Interfaces
            ("wasi", "http", _)
            | ("wasi", "blobstore", _)
            | ("wasi", "http", _)
            | ("wasi", "random", _)
            | ("wasmcloud", "messaging", _)
                if self.interfaces == vec!["types"] =>
            {
                None
            }
            ("wasi", "config", _) if self.interfaces == vec!["runtime"] => None,
            ("wasmcloud", "bus", _) => None,

            // Capability providers that implement well known WIT interfaces
            ("wasi", "blobstore", _) => Some("ghcr.io/wasmcloud/blobstore-fs:canary".to_string()),
            ("wasi", "http", Direction::Import) => {
                Some("ghcr.io/wasmcloud/http-client:canary".to_string())
            }
            ("wasi", "http", Direction::Export) => {
                Some("ghcr.io/wasmcloud/http-server:canary".to_string())
            }
            ("wasi", "keyvalue", _) => Some("ghcr.io/wasmcloud/keyvalue-redis:canary".to_string()),
            ("wasmcloud", "messaging", _) => {
                Some("ghcr.io/wasmcloud/messaging-nats:canary".to_string())
            }
            (namespace, package, _) => Some(format!(
                "REGISTRY-IMAGE/{}-{}-{}:{}",
                namespace,
                package,
                self.direction.as_str(),
                self.version.unwrap_or("latest")
            )),
        }
    }

    pub fn to_capability_component(&self) -> Option<Component> {
        self.capability_image().map(|image| Component {
            properties: Properties::Capability {
                properties: CapabilityProperties {
                    image,
                    id: None,
                    config: Vec::new(),
                },
            },
            name: self.name(),
            traits: None,
        })
    }

    pub fn to_source_link_property(&self, target: &str) -> Option<LinkProperty> {
        self.to_target_link_property().map(|link| LinkProperty {
            target: target.to_string(),
            ..link
        })
    }

    pub fn to_target_link_property(&self) -> Option<LinkProperty> {
        match (self.namespace, self.package) {
            // These interfaces are handled automatically in the host and do not have
            // associated mages
            // WASI Standard Interfaces, mostly in 0.2
            ("wasi", "logging")
            | ("wasi", "io")
            | ("wasi", "clocks")
            | ("wasi", "sockets")
            | ("wasi", "filesystem")
            | ("wasi", "cli") => None,
            // Types or Specific Interfaces
            ("wasi", "http")
            | ("wasi", "blobstore")
            | ("wasi", "http")
            | ("wasi", "random")
            | ("wasmcloud", "messaging")
                if self.interfaces == vec!["types"] =>
            {
                None
            }
            ("wasi", "config") if self.interfaces == vec!["runtime"] => None,
            ("wasmcloud", "bus") => None,
            _ => Some(LinkProperty {
                target: self.name(),
                namespace: self.namespace.to_string(),
                package: self.package.to_string(),
                interfaces: self.interfaces.iter().map(|s| s.to_string()).collect(),
                source_config: vec![],
                target_config: vec![],
                name: None,
            }),
        }
    }
}
