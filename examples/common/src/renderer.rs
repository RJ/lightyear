use bevy::prelude::*;

pub struct ExampleRendererPlugin;

impl Plugin for ExampleRendererPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor::default());
        // TODO common shortcuts for enabling the egui world inspector etc.
        // TODO handle bevygap ui things.
        // TODO for clients, provide a "connect" button?
        app.add_systems(Startup, spawn_text);
    }
}

fn spawn_text(mut commands: Commands) {
    commands.spawn(TextBundle::from_section(
        "This is the Example Renderer!",
        TextStyle::default(),
    ));
}
