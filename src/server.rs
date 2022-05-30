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
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

pub struct Server {
    dburl: String,
    pool: Pool,
    game: HashMap<String, Arc<RwLock<game::Game>>>,
    key: String,
    listen_url: String,
}

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
    PermissionDenied {}
}
pub struct ServerRequest {
    sn: Sender<ServerResponse>,
    dat: ServerRequestType,
    world : Option<String>, 
    session_token : Option<String>,
}

impl ServerRequest {
    pub fn handle(&self, response: ServerResponse) {
        self.sn.send(response);
    }
    pub fn new(dat: Value) -> Result<(Receiver<ServerResponse>, ServerRequest), serde_json::Error> {
        let op : Option<String> = match dat.get("world") {
            Some(val) => {
                val.as_str().map(Into::into)
            },
            None => {
                None
            },
        };

        let session_token : Option<String> = match dat.get("session_token") {
            Some(val) => {
                val.as_str().map(Into::into)
            },
            None => {
                None
            },
        };
        let (tx, rx) = crossbeam_channel::unbounded();
        Ok((
            rx,
            Self {
                sn: tx,
                dat: serde_json::from_value(dat)?,
                world : op,
                session_token : session_token
            },
        ))
    }
}
#[derive(Serialize, Deserialize)]
pub struct ServerClaims {
    pub user_name: String,
    pub is_admin : bool,
    pub exp : usize
}
pub struct ServerResponse {
    dat: ServerResponseType,
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
impl Server {
    fn initialize_database(&self) {
        let mut conn = self
            .get_conn()
            .expect("Error creating connection in database initialization");
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS worlds (
                world_id TEXT PRIMARY KEY NOT NULL)",
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
                chunk_id BIGINT UNSIGNED,
                world_id INT NOT NULL,
                chunk_dat BLOB,
                loaded BOOLEAN,
                FOREIGN KEY (world_id)
                    REFERENCES worlds(world_id)),
                PRIMARY KEY (chunk_id,world_id)",
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
    pub fn create_user(&self, username: &str, password: &str, is_admin : bool) -> bool {
        let mut conn = self.get_conn().expect("Could not get conn to create user");
        let pass = bcrypt::hash_with_result(password, 6).expect("Could not hash password");
        if !self.user_exists(username) {
            conn.exec_drop(
                "INSERT INTO users (user_name, password_hash, admin) VALUES (?,?, ?)",
                (username, pass.to_string(), is_admin),
            )
            .expect("Could not insert user");
            return true;
        }
        false
    }
    pub fn user_exists(&self, username: &str) -> bool {
        let mut conn = self.get_conn().expect("Could not get conn to create user");
        let res: Option<bool> = conn
            .exec_first(
                "SELECT admin FROM users WHERE user_name = :username",
                params! { "username" => username },
            )
            .expect("Error selecting user");
        res.is_some()
    }
    pub fn get_claims(&self, session : &str) -> Option<ServerClaims> {
        let token = decode::<ServerClaims>(session, &DecodingKey::from_secret(self.key.as_ref()), &Validation::default());
        match token {
            Ok(tok) => {
                Some(tok.claims)
            }
            Err(e) => {
                println!("{}",e);
                None
            }
        }
    }
    pub fn generate_session(&self, username: &str, password: &str) -> Option<String> {
        let mut conn = self
            .get_conn()
            .expect("Could not get conn to verify session");
        let user : Option<(String,bool)> = conn
            .exec_first(
                "SELECT password_hash, admin FROM users WHERE user_name = :username",
                params! {"username" => username},  
            )
            .expect("Could not execute query to verify session");
        match user {
            Some((u,is_admin)) => match bcrypt::verify(password, &u) {
                Ok(b) => {
                    if b {
                        let claims = ServerClaims {
                            user_name: String::from(username),
                            is_admin : is_admin,
                            exp: 10000000000
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
    fn listen_thread(listener: TcpListener, sv: Arc<RwLock<Self>>) {
        loop {
            for (mut conn, addr) in listener.accept() {
                //spawn a worker thread
                let svnew = sv.clone();
                std::thread::spawn(move || {
                    let mut buf = &mut [0; 1024];
                    conn.read(buf);
                    match std::str::from_utf8(buf) {
                        Ok(buf) => {
                            match serde_json::from_str::<Value>(buf.trim_end_matches(char::from(0)))
                            {
                                Ok(buf) => match ServerRequest::new(buf) {
                                    Ok((rx, req)) => {
                                        Self::worker_thread(req, svnew);
                                        match rx.recv_timeout(std::time::Duration::from_secs(1)) {
                                            Ok(dat) => {
                                                conn.write(
                                                    serde_json::to_string(&dat.dat)
                                                        .unwrap()
                                                        .as_bytes(),
                                                );
                                            }
                                            Err(_) => {
                                                conn.write(
                                                    serde_json::to_string(
                                                        &ServerResponseType::TimedOut {},
                                                    )
                                                    .unwrap()
                                                    .as_bytes(),
                                                );
                                                println!("Timed out");
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        println!("Not a valid server request")
                                    }
                                },
                                Err(_) => {
                                    println!("Not a valid json string")
                                }
                            }
                        }
                        Err(_) => {
                            println!("Not a valid utf-8 string")
                        },
                    }
                });
            }
        }
    }
    pub fn create_world(&mut self, world_name: &str) {
        let g = game::Game::new("./raws", self.get_conn().unwrap(), world_name.to_owned());
        let gmrwlock = Arc::new(RwLock::new(g));
        let gmrwlock2 = gmrwlock.clone();
        game::Game::start_game(gmrwlock);
        self.game.insert(String::from(world_name), gmrwlock2);
    }
    fn is_request_admin(&self, request: &ServerRequest) -> bool {
        match &request.session_token {
            Some(token) => {
                match self.get_claims(token) {
                    Some(claim) => {
                        claim.is_admin
                    }
                    None => {
                        false
                    }
                }
            }
            None => {
                false 
            }
        }
    }
    fn worker_thread(request: ServerRequest, sv: Arc<RwLock<Self>>) {
        match &request.dat {
            ServerRequestType::CreateGame { world_name } => {
                let guard0 = sv.read().unwrap();
                if guard0.is_request_admin(&request) {
                    drop(guard0);
                    let mut guard = sv.write().unwrap();
                    guard.create_world(&world_name);
                    request.handle(ServerResponse::new(ServerResponseType::Ok {}));
                } else {
                    request.handle(ServerResponse::new(ServerResponseType::PermissionDenied {  }));
                }
            }
            ServerRequestType::Login { user, password } => {
                let guard = sv.read().unwrap();
                match guard.generate_session(&user, &password) {
                    Some(token) => {
                        request.handle(ServerResponse::new(ServerResponseType::AuthSuccess {
                            session_token: token,
                        }));
                    }
                    None => {
                        request.handle(ServerResponse::new(ServerResponseType::AuthFailure {}));
                    }
                }
            }
            other => {

            }
        }
        match &request.world {
            Some(world_name) => {
                let guard = sv.read().unwrap();
                match guard.game.get(world_name) {
                    Some(gm) => {
                        let gmc = gm.clone();
                        game::Game::handle(gmc,request);
                    },
                    None => todo!(),
                }
            }
            None => {

            }
        }
    }
    pub fn new(args: &args::Args) -> Server {
        let (tx, rx) = crossbeam_channel::unbounded::<ServerRequest>();

        let key = args.secret.clone();
        let dburl = format!(
            "mysql://{}:{}@{}/{}",
            args.database_user, args.database_pass, args.database_host, args.database_name
        );
        println!("Connecting to database at {}", args.database_host);
        let ops = Opts::from_url(&dburl).expect("Database URL invalid");
        let pool = Pool::new(ops).expect("Could not establish database connection");
        println!("Connection establised");
        Self {
            dburl: dburl,
            listen_url: format!("{}:{}", args.ip, args.port),
            pool: pool,
            game: HashMap::new(),
            key: key,
        }
    }
    pub fn handle(&mut self, sr: &ServerRequest) {}
    pub fn get_conn(&self) -> Result<mysql::PooledConn, mysql::Error> {
        self.pool.get_conn()
    }
    pub fn run_game(self) {
        self.initialize_database();
        if !self.user_exists("admin") {
            println!("Creating user admin with default password \"password\"");
            self.create_user("admin", "password", true); 
        }
        let listener = TcpListener::bind(&self.listen_url).expect("Could not bind to ip/port");
        //create server arc
        let sv = Arc::new(RwLock::new(self));
        //create server listen thread
        std::thread::spawn(move || {
            Self::listen_thread(listener, sv.clone());
        });
        loop {}
    }
}

fn decode_valid_requests(stream: &TcpStream) -> Option<ServerRequest> {
    None
}
