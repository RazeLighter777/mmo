use crate::args;
use crate::game;
use bcrypt::bcrypt;
use crossbeam_channel::internal::SelectHandle;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use log::{info, warn};
use mysql::params;
use mysql::prelude::Queryable;
use mysql::Error;
use mysql::Opts;
use mysql::Pool;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Server {
    dburl: String,
    pool: Pool,
    game: Arc<Mutex<game::Game>>,
    key: EncodingKey,
}

pub enum ServerRequestType {
    LoadGame = 0,
    CloseGame = 1,
}
pub struct ServerRequest {
    sn: Sender<ServerResponse>,
    dat: Value,
}

impl ServerRequest {
    pub fn handle(&self, response: ServerResponse) {
        self.sn.send(response);
    }
    pub fn new(dat: Value) -> (Receiver<ServerResponse>, ServerRequest) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (rx, Self { sn: tx, dat: dat })
    }
}
#[derive(Serialize, Deserialize)]
struct ServerClaims {
    user_name: String,
}
pub struct ServerResponse {
    dat: Value,
}

impl ServerResponse {
    pub fn new(dat: Value) -> ServerResponse {
        Self { dat: dat }
    }
}
pub struct User {
    pub user_name: String,
    pub user_pass: String,
}
impl Server {
    fn initialize_database(&self) {
        let mut conn = self
            .get_conn()
            .expect("Error creating connection in database initialization");
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS worlds (
                world_id INT PRIMARY KEY NOT NULL AUTO_INCREMENT,
                world_name TEXT)",
        )
        .expect("Error creating world table");
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS users (
                user_id INT PRIMARY KEY NOT NULL AUTO_INCREMENT,
                user_name TEXT,
                password_hash TEXT,
                admin BOOLEAN)",
        )
        .expect("Error creating users table");
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS chunks (
                chunk_id BIGINT UNSIGNED PRIMARY KEY,
                world_id INT NOT NULL,
                chunk_dat BLOB,
                loaded BOOLEAN,
                FOREIGN KEY (world_id)
                    REFERENCES worlds(world_id))",
        )
        .expect("Error creating chunks table");
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS entities (
                entity_id BIGINT UNSIGNED PRIMARY KEY,
                chunk_id BIGINT UNSIGNED,
                world_id INT NOT NULL,
                FOREIGN KEY(chunk_id) 
                    REFERENCES chunks(chunk_id),
                FOREIGN KEY(world_id)
                    REFERENCES worlds(world_id)
                )",
        )
        .expect("Error creating entities table");
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS components (
                component_id BIGINT UNSIGNED PRIMARY KEY,
                type_id BIGINT UNSIGNED,
                dat TEXT,
                entity_id BIGINT UNSIGNED, 
                FOREIGN KEY(entity_id) 
                    REFERENCES entities(entity_id))",
        )
        .expect("Error creating components table");
    }
    pub fn create_user(&self, username: &str, password: &str) -> bool {
        let mut conn = self.get_conn().expect("Could not get conn to create user");
        let pass = bcrypt::hash_with_result(password, 10).expect("Could not hash password");
        if !self.user_exists(username) {
            conn.exec_drop(
                "INSERT INTO users (user_name, password_hash, admin) VALUES (?,?, true)",
                (username, pass.to_string()),
            )
            .expect("Could not insert user");
            return true;
        }
        false
    }
    pub fn user_exists(&self, username: &str) -> bool {
        let mut conn = self.get_conn().expect("Could not get conn to create user");
        let res: Option<bool> = conn
            .exec_first("SELECT admin FROM users WHERE user_name = :username", params! { "username" => username })
            .expect("Error selecting user");
        res.is_some()
    }
    pub fn generate_session(&self, username: &str, password: &str) -> Option<String> {
        let mut conn = self
            .get_conn()
            .expect("Could not get conn to verify session");
        let user: Option<String> = conn
            .query_first(
                "SELECT (password_hash) FROM users WHERE user_name = \'admin\'",
                )
            .expect("Could not execute query to verify session");
        user.as_ref().unwrap();
        match user {
            Some(u) => match bcrypt::verify(password, &u) {
                Ok(b) => {
                    if b {
                        let claims = ServerClaims {
                            user_name: String::from(username),
                        };
                        match encode(
                            &Header::default(),
                            &claims,
                            &EncodingKey::from_secret("secret".as_ref()),
                        ) {
                            Ok(tok) => return Some(tok),
                            Err(e) => {
                                return None;
                            }
                        }
                    }
                }

                Err(_) => {
                    println!("Failed sesssion verification attempt for {}", username);
                }
            },
            None => {
                println!("No such user found in database {}", username);
            }
        }
        None
    }
    pub fn new(args: &args::Args) -> Server {
        let key = EncodingKey::from_secret(args.secret.as_ref());
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
                                    match serde_json::from_str::<Value>(
                                        s.trim_end_matches(char::from(0)),
                                    ) {
                                        Ok(v) => {
                                            let (rx, req) = ServerRequest::new(v);
                                            let mut gl = gc2.lock();
                                            let mut gu = gl.unwrap();
                                            gu.handle(req);
                                            drop(gu);
                                            match rx.recv_timeout(std::time::Duration::from_secs(5))
                                            {
                                                Ok(dat) => {
                                                    stream.write(
                                                        serde_json::to_string(&dat.dat)
                                                            .expect("error writing response.")
                                                            .as_bytes(),
                                                    );
                                                }
                                                Err(e) => {
                                                    println!("Server did not respond to request.")
                                                }
                                            }
                                        }
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
            key: key,
        }
    }
    pub fn handle(&mut self, sr: &ServerRequest) {}
    pub fn get_conn(&self) -> Result<mysql::PooledConn, mysql::Error> {
        self.pool.get_conn()
    }
    pub fn run_game(&mut self) {
        self.initialize_database();
        if !self.user_exists("admin") {
            println!("Creating user admin with default password \"password\"");
            self.create_user("admin", "password");
        }
        loop {
            let mut lg = self.game.lock();
            let mut g = lg.unwrap();
            g.tick();
            drop(g);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
