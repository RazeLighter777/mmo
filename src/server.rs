use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;

use crate::args;
use crate::game;
use crossbeam_channel::internal::SelectHandle;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use log::{info, warn};
use mysql::Opts;
use mysql::Pool;
use serde_json::Value;
pub struct Server {
    dburl: String,
    pool: Pool,
    game: Arc<Mutex<game::Game>>,
}

pub enum ServerRequestType {
    LoadGame = 0,
    CloseGame = 1,
}
pub struct ServerRequest {
    sn: Sender<ServerResponse>,
    dat : Value
}

impl ServerRequest {
    pub fn handle(&self, response: ServerResponse) {
        self.sn.send(response);
    }
    pub fn new(dat : Value ) -> (Receiver<ServerResponse>, ServerRequest) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (rx, Self { sn: tx, dat : dat })
    }
}
pub struct ServerResponse {}

impl Server {
    pub fn new(args: &args::Args) -> Server {
        let dburl = format!(
            "mysql://{}:{}@{}/{}",
            args.database_user, args.database_pass, args.database_host, args.database_name
        );
        println!("Connecting to database at {}", args.database_host);
        let ops = Opts::from_url(&dburl).expect("Database URL invalid");
        let pool = Pool::new(ops).expect("Could not establish database connection");
        println!("Connection establised");
        let listener = TcpListener::bind(format!("{}:{}", args.ip, args.port))
            .expect("Could not bind to ip/port");
        //spawn the listener thread
        println!(
            "Listening on {}:{} for incoming connections",
            args.ip, args.port
        );
        let mut g = Arc::new(Mutex::new(game::Game::new()));
        let mut g2 = g.clone();
        let lt = std::thread::spawn(move || {
            let gc1 = g.clone();
            for c in listener.incoming() {
                match c {
                    Ok(mut stream) => {
                        let gc2 = gc1.clone();
                        std::thread::spawn(move || {
                            //let mut gu = gc2.lock().unwrap();
                            //gu.handle(req); // should be non blocking
                            let mut buf = &mut [0; 1024];
                            //stream.set_read_timeout(Some(std::time::Duration::from_secs(1)));
                            stream.read(buf);
                            match std::str::from_utf8(buf) {
                                Ok(s) => {
                                    println!("json {}, len {} ", s, s.len());
                                    match serde_json::from_str::<Value>(s.trim_end_matches(char::from(0))) {
                                        Ok(v) => { 
                                            let (rx, req) = ServerRequest::new(v);
                                            let mut gl = gc2.lock();
                                            let mut gu = gl.unwrap();
                                            gu.handle(req);
                                            drop(gu);
                                            match rx.recv_timeout(std::time::Duration::from_secs(5)) { 
                                            Ok(dat) => {
                                                println!("Valid json!");
                                            }
                                            Err(e) => {
                                                println!("Server did not respond to request.")
                                            }
                                        } },
                                        Err(e) => {
                                            println!("Invalid json!, {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("Stream wasn't UTF-8")
                                }
                            }
                            
                        });
                    }
                    Err(e) => {}
                }
            }
        });
        println!("Listening for inbound on addr {}:{}", args.ip, args.port);
        Self {
            dburl: dburl,
            pool: pool,
            game: g2,
        }
    }
    pub fn handle(&mut self, sr: &ServerRequest) {}
    pub fn get_conn(&self) -> Result<mysql::PooledConn, mysql::Error> {
        self.pool.get_conn()
    }
    pub fn run_game(&mut self) {
            loop { 
            let mut lg = self.game.lock();
            let mut g = lg.unwrap();
            g.tick();
            drop(g);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
