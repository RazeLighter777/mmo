
use std::sync::{Arc};
use tokio::{ net::TcpStream, sync::RwLock};
use tokio_tungstenite::{tungstenite::WebSocket, WebSocketStream};
use crossbeam_channel::{Receiver, Sender};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerRequestType {
    CreateGame { world_name: String },
    Login { user: String, password: String },
    Logout {},
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerResponseType {
    AuthSuccess { session_token: String },
    Ok {},
    AuthFailure {},
    TimedOut {},
    PermissionDenied {},
}
pub struct ServerRequest {
    sn: Sender<ServerResponse>,
    dat: ServerRequestType,
    world: Option<String>,
    session_token: Option<String>,
    claims: Option<TokenData<ServerClaims>>,
    connnection_lock : Arc<RwLock<WebSocketStream<TcpStream>>>
}

impl ServerRequest {
    pub fn handle(&self, response: ServerResponse) {
        self.sn.send(response);
    }
    pub fn new(
        dat: Value,
        secret_key: &str,
        connection_lock : Arc<RwLock<WebSocketStream<TcpStream>>>,
    ) -> Result<(Receiver<ServerResponse>, ServerRequest), serde_json::Error> {
        let op: Option<String> = match dat.get("world") {
            Some(val) => val.as_str().map(Into::into),
            None => None,
        };

        let session_token: Option<String> = match dat.get("session_token") {
            Some(val) => {
                let s = val.as_str().map(Into::into);
                s
            }
            None => None,
        };
        let claims = match &session_token {
            Some(s) => {
                match decode::<ServerClaims>(
                    &s,
                    &DecodingKey::from_secret(secret_key.as_bytes()),
                    &Validation::default(),
                ) {
                    Ok(claims) => Some(claims),
                    Err(_) => None,
                }
            }
            None => None,
        };
        let (tx, rx) = crossbeam_channel::unbounded();
        Ok((
            rx,
            Self {
                sn: tx,
                dat: serde_json::from_value(dat)?,
                world: op,
                session_token: session_token,
                claims: claims,
                connnection_lock : connection_lock
            },
        ))
    }
    pub fn get_user(&self) -> Option<&str> {
        match &self.claims {
            Some(claims) => Some(&claims.claims.user_name),
            None => None,
        }
    }
    pub fn is_admin(&self) -> bool {
        match &self.claims {
            Some(claims) => claims.claims.is_admin,
            None => false,
        }
    }
    pub fn get_dat(&self) -> &ServerRequestType {
        &self.dat
    }
    pub fn get_world(&self) -> &Option<String> {
        &self.world
    }
    pub fn get_connection(&self) -> Arc<RwLock<WebSocketStream<TcpStream>>> {
        self.connnection_lock.clone()
    }
}
#[derive(Serialize, Deserialize)]
pub struct ServerClaims {
    pub user_name: String,
    pub is_admin: bool,
    pub exp: usize,
}
pub struct ServerResponse {
    pub dat: ServerResponseType,
}

impl ServerResponse {
    pub fn new(dat: ServerResponseType) -> ServerResponse {
        Self { dat: dat }
    }
}
pub struct User {
    pub user_name: String,
    pub user_pass: String,
}
