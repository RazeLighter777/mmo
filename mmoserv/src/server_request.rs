use crossbeam_channel::{Receiver, Sender};
use futures::{stream::SplitSink, SinkExt};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use mmolib::{server_request_type::ServerRequestType, server_response_type::ServerResponseType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::Display, sync::Arc};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{
    tungstenite::{Message, WebSocket},
    WebSocketStream,
};
use tracing::{info, span, Level};

use crate::connection;

#[derive(Debug)]
pub struct ServerRequest {
    dat: ServerRequestType,
    world: Option<String>,
    session_token: Option<String>,
    claims: Option<TokenData<ServerClaims>>,
    connnection_lock: Arc<RwLock<SplitSink<WebSocketStream<TcpStream>, Message>>>,
}

impl ServerRequest {
    pub fn new(
        dat: Value,
        secret_key: &str,
        connection_lock: Arc<RwLock<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    ) -> Result<ServerRequest, serde_json::Error> {
        let op: Option<String> = match dat.get("world_name") {
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
        Ok(Self {
            dat: serde_json::from_value(dat)?,
            world: op,
            session_token: session_token,
            claims: claims,
            connnection_lock: connection_lock,
        })
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
    pub fn get_world(&self) -> Option<&str> {
        self.world.as_deref()
    }
    pub fn get_connection(&self) -> connection::Connection {
        connection::Connection::new(self.connnection_lock.clone(), self.get_user().unwrap_or(""))
    }
    pub async fn handle(self, request_dat: &ServerResponseType) {
        let lk = self.connnection_lock.write();
        let request_json = serde_json::to_string(request_dat).unwrap();
        info!("Sent response: {}", request_json);
        lk.await.send(Message::Text(request_json)).await;
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ServerClaims {
    pub user_name: String,
    pub is_admin: bool,
    pub exp: usize,
}
pub struct User {
    pub user_name: String,
    pub user_pass: String,
}

impl Display for ServerRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = std::format!(
            "REQUEST[user:{},world:{},dat:{:?}]",
            self.get_user().unwrap_or("anonymous"),
            self.get_world().unwrap_or("null"),
            self.get_dat()
        );
        write!(f, "{}", r)
    }
}
