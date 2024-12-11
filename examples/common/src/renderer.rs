use bevy::prelude::*;

pub struct ExampleRendererPlugin;

impl Plugin for ExampleRendererPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor::default());
        // TODO common shortcuts for enabling the egui world inspector etc.
        // TODO handle bevygap ui things.
        // TODO for clients, provide a "connect" button?
        app.add_systems(Startup, spawn_text);

        #[cfg(feature = "server")]
        app.add_systems(Startup, spawn_server_text);
    }
}

fn spawn_text(mut commands: Commands) {
    commands.spawn(TextBundle::from_section(
        "This is the Example Renderer!",
        TextStyle::default(),
    ));
}

/// Spawns a text element that displays "Server"
fn spawn_server_text(mut commands: Commands) {
    commands.spawn(
        TextBundle::from_section(
            "Server",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::End,
            ..default()
        }),
    );
}
