use anyhow::Context;
use async_compat::Compat;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use bevy::prelude::*;
use bevy::utils::{Duration, HashSet};
use bevy::{prelude::*, tasks::IoTaskPool};
use lightyear::connection::netcode::{self, USER_DATA_BYTES};
use lightyear::server::events::{ConnectEvent, DisconnectEvent};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use tokio::io::AsyncWriteExt;

use base64::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::ClientId::Netcode;
use lightyear::prelude::*;

pub struct TokenServer {
    pub netcode_params: NetcodeParams,
}

impl Plugin for TokenServer {
    fn build(&self, app: &mut App) {
        let www_state = NetcodeState::new(self.netcode_params.clone());
        app.insert_resource(www_state.clone());
        app.add_systems(Update, handle_connect_events);
        start_httpd(www_state);
    }
}

fn start_httpd(www_state: NetcodeState) {
    IoTaskPool::get()
        .spawn(Compat::new(async move {
            let app = Router::new()
                .route("/", get(www_root))
                .route("/token-please", post(www_token_please))
                .with_state(www_state);

            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
                .await
                .expect("binding http port");
            info!("Token server listening on port 3000");
            axum::serve(listener, app).await.unwrap();
        }))
        .detach();
}

fn handle_connect_events(
    mut connect_events: EventReader<ConnectEvent>,
    mut disconnect_events: EventReader<DisconnectEvent>,
    mut netstate: ResMut<NetcodeState>,
) {
    for event in connect_events.read() {
        if let Netcode(client_id) = event.client_id {
            netstate.client_connected(client_id);
        }
    }
    for event in disconnect_events.read() {
        if let Netcode(client_id) = event.client_id {
            netstate.client_disconnected(client_id);
        }
    }
}

#[derive(Deserialize)]
struct TokenRequest {
    name: String,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

async fn www_root(State(_netstate): State<NetcodeState>) -> &'static str {
    "Hello"
}

async fn www_token_please(
    State(mut netstate): State<NetcodeState>,
    Json(payload): Json<TokenRequest>,
) -> (StatusCode, Json<TokenResponse>) {
    info!("Issuing token for {:?}", payload.name);
    if let Ok(token) = netstate.issue_token(payload.name) {
        let serialized_token = token.try_into_bytes().expect("Failed to serialize token");
        let token_string = BASE64_STANDARD.encode(serialized_token);
        let response = TokenResponse {
            token: token_string,
        };
        (StatusCode::OK, Json(response))
    } else {
        let response = TokenResponse {
            token: "Failed to issue token".to_owned(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
    }
}

#[derive(Resource, Clone)]
pub struct NetcodeState {
    pub netcode_params: Arc<Mutex<NetcodeParams>>,
}
impl NetcodeState {
    pub fn new(netcode_params: NetcodeParams) -> Self {
        Self {
            netcode_params: Arc::new(Mutex::new(netcode_params)),
        }
    }

    pub fn address_protocol_key(&self) -> (SocketAddr, u64, Key) {
        let params = self.netcode_params.lock().unwrap();
        (
            params.game_server_addr,
            params.protocol_id,
            params.private_key,
        )
    }

    pub fn issue_token(&mut self, name: String) -> Result<ConnectToken, ()> {
        let params = self.netcode_params.lock().unwrap();
        let client_id = loop {
            let client_id = rand::random();
            if !params.client_ids.contains(&client_id) {
                break client_id;
            }
        };
        let user_data = string_to_user_data(crate::name_generator::sanitise_name(client_id, name))
            .unwrap_or([0u8; USER_DATA_BYTES]);
        Ok(ConnectToken::build(
            params.game_server_addr,
            params.protocol_id,
            client_id,
            params.private_key,
        )
        .expire_seconds(60)
        .user_data(user_data)
        .generate()
        .expect("Failed to generate token"))

        // let serialized_token = token.try_into_bytes().expect("Failed to serialize token");
    }

    pub fn client_connected(&mut self, client_id: u64) {
        let mut params = self.netcode_params.lock().unwrap();
        params.client_ids.insert(client_id);
        info!("client connected {client_id}");
    }

    pub fn client_disconnected(&mut self, client_id: u64) {
        let mut params = self.netcode_params.lock().unwrap();
        params.client_ids.remove(&client_id);
        info!("client disconnected {client_id}");
    }
}

/// copy string into fixed len array for netcode user data field
fn string_to_user_data(input: String) -> Result<[u8; USER_DATA_BYTES], ()> {
    let mut output = [0u8; USER_DATA_BYTES];
    let bytes = input.into_bytes();
    if bytes.len() > USER_DATA_BYTES {
        Err(())
    } else {
        let safe_len = std::cmp::min(bytes.len(), USER_DATA_BYTES);
        output[..safe_len].copy_from_slice(&bytes[..safe_len]);
        Ok(output)
    }
}

#[derive(Clone)]
pub struct NetcodeParams {
    pub protocol_id: u64,
    pub private_key: Key,
    pub game_server_addr: SocketAddr,
    pub client_ids: HashSet<u64>,
}
