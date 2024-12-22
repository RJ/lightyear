#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use crate::shared::SharedPlugin;
use bevy::prelude::*;
use lightyear_examples_common::app::{Apps, Cli};
use lightyear_examples_common::settings::{read_settings, Settings};

#[cfg(feature = "client")]
mod client;
mod protocol;

#[cfg(feature = "gui")]
mod renderer;
#[cfg(feature = "server")]
mod server;
mod shared;

fn main() {
    let cli = Cli::default();
    let settings_str = include_str!("../assets/settings.ron");
    let settings = read_settings::<Settings>(settings_str);
    // build the bevy app (this adds common plugin such as the DefaultPlugins)
    // and returns the `ClientConfig` and `ServerConfig` so that we can modify them if needed
    let mut apps = Apps::new(settings, cli, env!("CARGO_PKG_NAME").to_string());
    apps.add_lightyear_plugins();
    apps.add_user_shared_plugin(SharedPlugin);
    #[cfg(feature = "client")]
    apps.add_user_client_plugin(crate::client::ExampleClientPlugin);
    #[cfg(feature = "server")]
    apps.add_user_server_plugin(crate::server::ExampleServerPlugin);
    #[cfg(feature = "gui")]
    apps.add_user_renderer_plugin(crate::renderer::ExampleRendererPlugin);

    // run the app
    apps.run();
}
