use futures::{future, pin_mut, StreamExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub struct Connection {
    ws: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}
impl Connection {
    pub async fn new(url: &str) -> Result<Self, tokio_tungstenite::tungstenite::Error> {
        Ok(Self {
            ws: Some(tokio_tungstenite::connect_async(url).await?.0),
        })
    }
    pub fn new_dont_start() -> Self {
        Self { ws: None }
    }
    pub fn get(&mut self) -> Option<&mut WebSocketStream<MaybeTlsStream<TcpStream>>> {
        self.ws.as_mut()
    }
}
