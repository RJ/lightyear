use base64::prelude::*;
use bevy::{ecs::system::Command, prelude::*};
use bevy_http_client::prelude::*;
use lightyear::{
    connection::{
        client::{Authentication, NetConfig},
        netcode::ConnectToken,
    },
    prelude::client::ClientCommands,
};
use serde::{Deserialize, Serialize};

use crate::token_server::{TokenRequest, TokenResponse};

#[allow(clippy::large_enum_variant)]
#[derive(Event, Clone)]
pub enum ConnectTokenResult {
    Ok(ConnectToken),
    ServerError,
}

pub struct TokenClientPlugin;

impl Plugin for TokenClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HttpClientPlugin);
        app.add_event::<ConnectTokenResult>();
        app.register_request_type::<TokenResponse>();
        app.add_systems(
            Update,
            (
                handle_response.run_if(
                    on_event::<TypedResponse<TokenResponse>>()
                        .or_else(on_event::<TypedResponseError<TokenResponse>>()),
                ),
                connect_with_token.run_if(on_event::<ConnectTokenResult>()),
            ),
        );
    }
}
fn connect_with_token(
    mut commands: Commands,
    mut client_config: ResMut<lightyear::client::config::ClientConfig>,
    mut ev: EventReader<ConnectTokenResult>,
) {
    for ct in ev.read() {
        match ct {
            ConnectTokenResult::Ok(connect_token) => {
                // if we have received the connect token, update the `ClientConfig` to use it to connect
                // to the game server
                if let NetConfig::Netcode { auth, .. } = &mut client_config.net {
                    *auth = Authentication::Token(connect_token.clone());
                }
                info!("Connecting with connect token...");
                commands.connect_client();
            }
            ConnectTokenResult::ServerError => {
                warn!("Server error when fetching connect token");
            }
        }
    }
}

fn handle_response(
    mut ev_response: EventReader<TypedResponse<TokenResponse>>,
    mut ev_response_err: EventReader<TypedResponseError<TokenResponse>>,
    mut ev_ct: EventWriter<ConnectTokenResult>,
) {
    for response_err in ev_response_err.read() {
        warn!("err: {response_err:?}");
        ev_ct.send(ConnectTokenResult::ServerError);
    }
    for response in ev_response.read() {
        info!("ok: {response:?}");
        if let Ok(tok) = BASE64_STANDARD.decode(response.token.clone()) {
            if let Ok(connect_token) = ConnectToken::try_from_bytes(tok.as_slice()) {
                ev_ct.send(ConnectTokenResult::Ok(connect_token));
                continue;
            }
        }
        ev_ct.send(ConnectTokenResult::ServerError);
    }
}

struct RequestConnectTokenCommand {
    auth_backend_uri: String,
}

impl Command for RequestConnectTokenCommand {
    fn apply(self, world: &mut World) {
        info!("Sending token please request");
        world.resource_scope(
            |_world, mut ev_req: Mut<Events<TypedRequest<TokenResponse>>>| {
                let client_req = HttpClient::new()
                    .post(self.auth_backend_uri)
                    .json(&TokenRequest {
                        name: "RJ".to_owned(),
                    })
                    .with_type::<TokenResponse>();

                ev_req.send(client_req);
            },
        );
    }
}

pub trait RequestConnectTokenExt {
    fn request_connect_token_and_connect(self, auth_backend_uri: &str);
}

impl<'w, 's> RequestConnectTokenExt for Commands<'w, 's> {
    fn request_connect_token_and_connect(mut self, auth_backend_uri: &str) {
        self.add(RequestConnectTokenCommand {
            auth_backend_uri: auth_backend_uri.into(),
        });
    }
}
