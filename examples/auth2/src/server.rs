//! The server side of the example.
//! It is possible (and recommended) to run the server in headless mode (without any rendering plugins).
//!
//! The server will:
//! - spawn a new player entity for each client that connects
//! - read inputs from the clients and move the player entities accordingly
//!
//! Lightyear will handle the replication of entities automatically if you add a `Replicate` component to them.
use anyhow::Context;
use async_compat::Compat;
use lightyear::connection::netcode::{self, USER_DATA_BYTES};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};

use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy::utils::{Duration, HashSet};
use tokio::io::AsyncWriteExt;

use lightyear::prelude::server::*;
use lightyear::prelude::ClientId::Netcode;
use lightyear::prelude::*;

use serde::{Deserialize, Serialize};

use crate::protocol::*;
use crate::shared;

use crate::token_server::*;
pub struct ExampleServerPlugin {
    pub(crate) netcode_params: NetcodeParams,
}

impl Plugin for ExampleServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TokenServer {
            netcode_params: self.netcode_params.clone(),
        });
        app.add_systems(Startup, (init, start_server));
    }
}

/// Start the server
fn start_server(mut commands: Commands) {
    commands.start_server();
}

/// Add some debugging text to the screen
fn init(mut commands: Commands) {
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

// curl -H "Content-type: application/json" -d '{"name": "rj"}' http://localhost:3000/token-please
