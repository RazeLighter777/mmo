use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;

use crate::args;
use crate::game;
use log::{info, warn};
use mysql::Opts;
use mysql::Pool;
pub struct Server {
    dburl: String,
    pool: Pool,
    game: Arc<Mutex<game::Game>>,
}

pub struct ServerRequest {}

impl ServerRequest {}
pub struct ServerResponse {}

impl Server {
    pub fn new(args: &args::Args) -> Server {
        let dburl = format!(
            "mysql://{}:{}@{}/{}",
            args.database_user, args.database_pass, args.database_host, args.database_name
        );
        println!("Connecting to database at {}", args.database_host);
        let ops = Opts::from_url(&dburl).expect("Database URL invalid");
        let pool  = Pool::new(ops).expect("Could not establish database connection");
        println!("Connection establised");
        let listener = TcpListener::bind(format!("{}:{}", args.ip, args.port)).expect("Could not bind to ip/port");
        //spawn the listener thread
        println!("Listening on {}:{} for incoming connections", args.ip, args.port);
        let mut g = Arc::new(Mutex::new(game::Game::new()));
        let mut g2 = g.clone();
        let lt = std::thread::spawn(move || {
            let gc1 = g.clone();
            for c in listener.incoming() {
                std::thread::spawn(move || {

                });
            }
        });
        println!("Listening for inbound on addr {}:{}", args.ip, args.port);
        Self {
            dburl: dburl,
            pool: pool,
            game: g2,
        }
    }
}
