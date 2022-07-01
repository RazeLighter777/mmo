use std::sync::Arc;

use futures::{stream::SplitSink, SinkExt};
use mmolib::entity_id::EntityId;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[derive(Clone)]
pub struct Connection {
    username: String,
    active_connection: Arc<tokio::sync::RwLock<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    player: Option<EntityId>,
}

impl Connection {
    pub fn new(
        active_connection: Arc<tokio::sync::RwLock<SplitSink<WebSocketStream<TcpStream>, Message>>>,
        username: &str,
    ) -> Self {
        Self {
            username: username.to_owned(),
            active_connection: active_connection,
            player: None,
        }
    }
    pub fn close(self) {}
    #[inline(never)]
    pub async fn send(
        &self,
        response: mmolib::server_response_type::ServerResponseType,
    ) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        let mut lk = self.active_connection.write().await;
        lk.send(Message::Binary(serde_cbor::to_vec(&response).unwrap()))
            .await?;
        Ok(())
    }
    pub fn get_player(&self) -> Option<EntityId> {
        self.player
    }
    pub fn get_username(&self) -> &str {
        &self.username
    }
    pub fn set_player(&mut self, player: EntityId) {
        self.player = Some(player);
    }
}
