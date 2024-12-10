//! This module parses the settings.ron file and builds a lightyear configuration from it
#![allow(unused_imports)]
#![allow(unused_variables)]
use std::net::{Ipv4Addr, SocketAddr};

use bevy::asset::ron;
use bevy::prelude::{default, Resource};
use bevy::utils::Duration;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[cfg(not(target_family = "wasm"))]
use async_compat::Compat;
#[cfg(not(target_family = "wasm"))]
use bevy::tasks::IoTaskPool;

use lightyear::prelude::client::Authentication;
#[cfg(feature = "steam")]
use lightyear::prelude::client::{SocketConfig, SteamConfig};
use lightyear::prelude::{CompressionConfig, LinkConditionerConfig};

use lightyear::prelude::{client, server};

/// We parse the settings.ron file to read the settings
pub fn read_settings<T: DeserializeOwned>(settings_str: &str) -> T {
    ron::de::from_str::<T>(settings_str).expect("Could not deserialize the settings file")
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ClientTransports {
    #[cfg(not(target_family = "wasm"))]
    Udp,
    WebTransport {
        certificate_digest: String,
    },
    WebSocket,
    #[cfg(feature = "steam")]
    Steam {
        app_id: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ServerTransports {
    Udp {
        local_port: u16,
    },
    WebTransport {
        local_port: u16,
        certificate: WebTransportCertificateSettings,
    },
    WebSocket {
        local_port: u16,
    },
    #[cfg(feature = "steam")]
    Steam {
        app_id: u32,
        server_ip: Ipv4Addr,
        game_port: u16,
        query_port: u16,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Conditioner {
    /// One way latency in milliseconds
    pub(crate) latency_ms: u16,
    /// One way jitter in milliseconds
    pub(crate) jitter_ms: u16,
    /// Percentage of packet loss
    pub(crate) packet_loss: f32,
}

impl Conditioner {
    pub fn build(&self) -> LinkConditionerConfig {
        LinkConditionerConfig {
            incoming_latency: Duration::from_millis(self.latency_ms as u64),
            incoming_jitter: Duration::from_millis(self.jitter_ms as u64),
            incoming_loss: self.packet_loss,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerSettings {
    /// If true, disable any rendering-related plugins
    pub(crate) headless: bool,

    /// If true, enable bevy_inspector_egui
    pub(crate) inspector: bool,

    /// Possibly add a conditioner to simulate network conditions
    pub(crate) conditioner: Option<Conditioner>,

    /// Which transport to use
    pub transport: Vec<ServerTransports>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientSettings {
    /// If true, enable bevy_inspector_egui
    pub(crate) inspector: bool,

    /// The client id
    pub(crate) client_id: u64,

    /// The client port to listen on
    pub(crate) client_port: u16,

    /// The ip address of the server
    pub server_addr: Ipv4Addr,

    /// The port of the server
    pub server_port: u16,

    /// Which transport to use
    pub(crate) transport: ClientTransports,

    /// Possibly add a conditioner to simulate network conditions
    pub(crate) conditioner: Option<Conditioner>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct SharedSettings {
    /// An id to identify the protocol version
    pub protocol_id: u64,

    /// a 32-byte array to authenticate via the Netcode.io protocol
    pub private_key: [u8; 32],

    /// compression options
    pub(crate) compression: CompressionConfig,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub client: ClientSettings,
    pub shared: SharedSettings,
}

#[allow(dead_code)]
pub(crate) fn build_server_netcode_config(
    conditioner: Option<&Conditioner>,
    shared: &SharedSettings,
    transport_config: server::ServerTransport,
) -> server::NetConfig {
    let conditioner = conditioner.map(|c| LinkConditionerConfig {
        incoming_latency: Duration::from_millis(c.latency_ms as u64),
        incoming_jitter: Duration::from_millis(c.jitter_ms as u64),
        incoming_loss: c.packet_loss,
    });
    let netcode_config = server::NetcodeConfig::default()
        .with_protocol_id(shared.protocol_id)
        .with_key(shared.private_key);
    let io_config = server::IoConfig {
        transport: transport_config,
        conditioner,
        compression: shared.compression,
    };
    server::NetConfig::Netcode {
        config: netcode_config,
        io: io_config,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum WebTransportCertificateSettings {
    /// Lightyear will generate a self-signed certificate, with given SANs list.
    AutoSelfSigned(Vec<String>),
    /// Lightyear will load certificate pem files from disk
    FromFile {
        cert_pem_path: String,
        private_key_pem_path: String,
    },
}

impl Default for WebTransportCertificateSettings {
    fn default() -> Self {
        let sans = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
        ];
        WebTransportCertificateSettings::AutoSelfSigned(sans)
    }
}

impl From<&WebTransportCertificateSettings> for server::Identity {
    fn from(wt: &WebTransportCertificateSettings) -> server::Identity {
        match wt {
            WebTransportCertificateSettings::AutoSelfSigned(sans) => {
                println!("🔐 Creating self-signed certificate with SANs: {:?}", sans);
                server::Identity::self_signed(sans).unwrap()
            }
            WebTransportCertificateSettings::FromFile {
                cert_pem_path,
                private_key_pem_path,
            } => {
                // this is async because we need to load the certificate from io
                // we need async_compat because wtransport expects a tokio reactor
                let identity = IoTaskPool::get()
                    .scope(|s| {
                        s.spawn(Compat::new(async {
                            server::Identity::load_pemfiles(cert_pem_path, private_key_pem_path)
                                .await
                                .unwrap()
                        }));
                    })
                    .pop()
                    .unwrap();
                let digest = identity.certificate_chain().as_slice()[0].hash();
                println!("Generated self-signed certificate with digest: {}", digest);
                identity
            }
        }
    }
}

/// Parse the settings into a list of `NetConfig` that are used to configure how the lightyear server
/// listens for incoming client connections
#[cfg(not(target_family = "wasm"))]
pub(crate) fn get_server_net_configs(settings: &Settings) -> Vec<server::NetConfig> {
    settings
        .server
        .transport
        .iter()
        .map(|t| match t {
            ServerTransports::Udp { local_port } => build_server_netcode_config(
                settings.server.conditioner.as_ref(),
                &settings.shared,
                server::ServerTransport::UdpSocket(SocketAddr::new(
                    Ipv4Addr::UNSPECIFIED.into(),
                    *local_port,
                )),
            ),
            ServerTransports::WebTransport {
                local_port,
                certificate,
            } => {
                let transport_config = server::ServerTransport::WebTransportServer {
                    server_addr: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), *local_port),
                    certificate: certificate.into(),
                };
                build_server_netcode_config(
                    settings.server.conditioner.as_ref(),
                    &settings.shared,
                    transport_config,
                )
            }
            ServerTransports::WebSocket { local_port } => build_server_netcode_config(
                settings.server.conditioner.as_ref(),
                &settings.shared,
                server::ServerTransport::WebSocketServer {
                    server_addr: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), *local_port),
                },
            ),
            #[cfg(feature = "steam")]
            ServerTransports::Steam {
                app_id,
                server_ip,
                game_port,
                query_port,
            } => server::NetConfig::Steam {
                steamworks_client: None,
                config: server::SteamConfig {
                    app_id: *app_id,
                    socket_config: server::SocketConfig::Ip {
                        server_ip: *server_ip,
                        game_port: *game_port,
                        query_port: *query_port,
                    },
                    max_clients: 16,
                    ..default()
                },
                conditioner: settings.server.conditioner.as_ref().map(|c| c.build()),
            },
        })
        .collect()
}

/// Build a netcode config for the client
pub(crate) fn build_client_netcode_config(
    client_id: u64,
    server_addr: SocketAddr,
    conditioner: Option<&Conditioner>,
    shared: &SharedSettings,
    transport_config: client::ClientTransport,
) -> client::NetConfig {
    let conditioner = conditioner.map(|c| c.build());
    let auth = Authentication::Manual {
        server_addr,
        client_id,
        private_key: shared.private_key,
        protocol_id: shared.protocol_id,
    };
    let netcode_config = client::NetcodeConfig::default();
    let io_config = client::IoConfig {
        transport: transport_config,
        conditioner,
        compression: shared.compression,
    };
    client::NetConfig::Netcode {
        auth,
        config: netcode_config,
        io: io_config,
    }
}

/// Parse the settings into a `NetConfig` that is used to configure how the lightyear client
/// connects to the server
pub fn get_client_net_config(settings: &Settings, client_id: u64) -> client::NetConfig {
    let server_addr = SocketAddr::new(
        settings.client.server_addr.into(),
        settings.client.server_port,
    );
    let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client.client_port);
    match &settings.client.transport {
        #[cfg(not(target_family = "wasm"))]
        ClientTransports::Udp => build_client_netcode_config(
            client_id,
            server_addr,
            settings.client.conditioner.as_ref(),
            &settings.shared,
            client::ClientTransport::UdpSocket(client_addr),
        ),
        ClientTransports::WebTransport { certificate_digest } => build_client_netcode_config(
            client_id,
            server_addr,
            settings.client.conditioner.as_ref(),
            &settings.shared,
            client::ClientTransport::WebTransportClient {
                client_addr,
                server_addr,
                #[cfg(target_family = "wasm")]
                certificate_digest: certificate_digest.to_string().replace(":", ""),
            },
        ),
        ClientTransports::WebSocket => build_client_netcode_config(
            client_id,
            server_addr,
            settings.client.conditioner.as_ref(),
            &settings.shared,
            client::ClientTransport::WebSocketClient { server_addr },
        ),
        #[cfg(feature = "steam")]
        ClientTransports::Steam { app_id } => client::NetConfig::Steam {
            steamworks_client: None,
            config: SteamConfig {
                socket_config: SocketConfig::Ip { server_addr },
                app_id: *app_id,
            },
            conditioner: settings.server.conditioner.as_ref().map(|c| c.build()),
        },
    }
}
