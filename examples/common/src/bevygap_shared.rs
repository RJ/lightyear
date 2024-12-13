/// Registers a channel and replicated resource for server metadata.
/// This is used to tell the client info about the server they've connected to.
use bevy::prelude::*;
#[cfg(feature = "bevygap_server")]
use bevygap_server_plugin::prelude::*;
use lightyear::prelude::*;

#[derive(Clone)]
pub struct BevygapSharedExtensionPlugin;

impl Plugin for BevygapSharedExtensionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServerMetadata>();
        app.register_resource::<ServerMetadata>(ChannelDirection::ServerToClient);
        app.add_channel::<ServerMetadataChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..default()
        });

        #[cfg(feature = "bevygap_server")]
        app.add_systems(
            Update,
            update_server_metadata.run_if(resource_added::<ArbitriumContext>),
        );

        #[cfg(feature = "bevygap_client")]
        app.add_systems(
            Update,
            on_server_metadata_changed.run_if(resource_changed::<ServerMetadata>),
        );
    }
}

/// Used to replicate ServerMetadata resource
#[derive(Channel)]
pub struct ServerMetadataChannel;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct ServerMetadata {
    pub location: String,
    pub fqdn: String,
    pub build_info: String,
}

impl Default for ServerMetadata {
    fn default() -> Self {
        Self {
            location: "unknown location".to_string(),
            fqdn: "unknown.example.com".to_string(),
            build_info: "".to_string(),
        }
    }
}

#[cfg(feature = "bevygap_client")]
fn on_server_metadata_changed(metadata: ResMut<ServerMetadata>) {
    info!("Server metadata changed: {metadata:?}");
}

#[cfg(feature = "bevygap_server")]
fn update_server_metadata(
    mut metadata: ResMut<ServerMetadata>,
    context: Res<ArbitriumContext>,
    mut commands: Commands,
) {
    metadata.fqdn = context.fqdn();
    metadata.location = context.location();
    info!("Updating server metadata: {metadata:?}");
    commands.replicate_resource::<ServerMetadata, ServerMetadataChannel>(NetworkTarget::All);
}
