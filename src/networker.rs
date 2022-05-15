use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    thread::JoinHandle,
};

use crate::generator;
use crossbeam_channel::{unbounded, Receiver};
use log::{info, warn};
pub struct Networker {
    handle: JoinHandle<()>,
    rx: Receiver<TcpStream>,
}

impl generator::Generator for Networker {
    fn update(&mut self) -> () {
        for mut stream in &self.rx {
            stream.write("hello".as_bytes());
        }
    }

    fn generate(
        &self,
        world: std::sync::Arc<std::sync::Mutex<&crate::world::World>>,
        ents: &Vec<crate::entity::EntityId>,
    ) -> Vec<Box<dyn crate::game_event::GameEventInterface>> {
        Vec::new()
    }

    fn request(&self) -> Vec<crate::component::ComponentTypeId> {
        Vec::new()
    }
}
impl Networker {
    pub fn new(host: &str, port: u16) -> Self {
        let mut listener =
            TcpListener::bind(format!("{}:{}", host, port)).expect("Could not bind to address");
        let (tx, rx) = unbounded();

        Self {
            rx: rx,
            handle: std::thread::spawn(move || {
                let thread_tx = tx.clone();
                for stream in listener.incoming() {
                    match stream {
                        Ok(mut res) => {
                            match res.peer_addr() {
                                Ok(r) => {
                                    println!("Connection recieved from {}", r.to_string());
                                }
                                Err(r) => {}
                            }
                            thread_tx.send(res);
                            //res.shutdown(Shutdown::Both);
                        }
                        Err(er) => {
                            warn!("Error in stream: {}", er);
                        }
                    }
                }
            }),
        }
    }
}
