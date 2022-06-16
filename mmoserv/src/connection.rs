use std::sync::Arc;

use futures::{stream::SplitSink, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub struct Connection {
    username: String,
    active_connection: Arc<tokio::sync::RwLock<SplitSink<WebSocketStream<TcpStream>, Message>>>,
}

impl Connection {
    pub fn new(
        active_connection: Arc<tokio::sync::RwLock<SplitSink<WebSocketStream<TcpStream>, Message>>>,
        username: &str,
    ) -> Self {
        Self {
            username: username.to_owned(),
            active_connection: active_connection,
        }
    }
    pub fn close(self) {}
    #[inline(never)]
    pub async fn send(&self, response: mmolib::server_response_type::ServerResponseType) {
        let mut lk = self.active_connection.write().await;
        println!("Sent message");
        lk.send(Message::Text(
            serde_json::to_string(&response)
                .unwrap()
                .as_str()
                .to_owned(),
        ))
        .await;
    }
    pub async fn is_closed(&self) -> bool {
        false
    }
}
