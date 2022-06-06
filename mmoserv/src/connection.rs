use std::sync::Arc;

use futures::SinkExt;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

pub struct Connection {
    active_connection : Arc<tokio::sync::RwLock<WebSocketStream<tokio::net::TcpStream>>>
}

impl Connection {
    pub fn new(active_connection : Arc<tokio::sync::RwLock<WebSocketStream<tokio::net::TcpStream>>>) -> Self {
        Self { active_connection: active_connection }
    }
    pub fn close(self) {
    }
    pub async fn send(&self, response : mmolib::server_response_type::ServerResponseType) {
        let mut lk = self.active_connection.write().await;
        lk.send(Message::Text(
            serde_json::to_string(&response)
                .unwrap()
                .as_str()
                .to_owned(),
        )).await;
    }
}