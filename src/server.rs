use crate::args;
use crate::game;
use async_std::prelude::FutureExt;
use bcrypt::bcrypt;
use crossbeam_channel::internal::SelectHandle;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use jsonwebtoken::TokenData;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use log::{info, warn};
use async_std::task;

use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use sqlx::Acquire;
use sqlx::MySql;
use sqlx::Pool;
use sqlx::Row;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::mysql::MySqlRow;
use sqlx::pool::PoolConnection;
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
    pool: Pool<MySql>,
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
    claims : Option<TokenData<ServerClaims>>,
}

impl ServerRequest {
    pub fn handle(&self, response: ServerResponse) {
        self.sn.send(response);
    }
    pub fn new(dat: Value, secret_key : &str) -> Result<(Receiver<ServerResponse>, ServerRequest), serde_json::Error> {
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
                let s = val.as_str().map(Into::into);
                s
            },
            None => {
                None
            },
        };
        let claims = match &session_token {
            Some(s) => {  
                match decode::<ServerClaims>(&s, &DecodingKey::from_secret(secret_key.as_bytes()), &Validation::default()) {
                    Ok(claims) => {
                        Some(claims)
                    },
                    Err(_) => {
                        None
                    }
                }
            },
            None => None,
        };
        let (tx, rx) = crossbeam_channel::unbounded();
        Ok((
            rx,
            Self {
                sn: tx,
                dat: serde_json::from_value(dat)?,
                world : op,
                session_token : session_token,
                claims : claims
            },
        ))
    }
    pub fn get_user(&self) -> Option<&str> {
        match &self.claims {
            Some(claims) => {
                Some(&claims.claims.user_name)
            },
            None => {
                None
            },
        }
    }
    pub fn is_admin(&self) -> bool {
        match &self.claims {
            Some(claims) => {
                claims.claims.is_admin
            },
            None => {
                false
            },
        }
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
    async fn initialize_database(&self) {
        sqlx::query(
            r"CREATE TABLE IF NOT EXISTS worlds (
                world_id VARCHAR(50) PRIMARY KEY NOT NULL)",
            ).execute(&self.pool).await.unwrap();
        sqlx::query(
            r"CREATE TABLE IF NOT EXISTS users (
                user_id INT PRIMARY KEY NOT NULL AUTO_INCREMENT,
                user_name TEXT,
                password_hash TEXT,
                admin BOOLEAN)",
        ).execute(&self.pool).await.unwrap();
        sqlx::query(
            r"CREATE TABLE IF NOT EXISTS chunks (
                chunk_id BIGINT UNSIGNED,
                world_id VARCHAR(50)  NOT NULL,
                chunk_dat BLOB,
                loaded BOOLEAN,
                FOREIGN KEY (world_id)
                    REFERENCES worlds(world_id),
                PRIMARY KEY (chunk_id,world_id))",
        ).execute(&self.pool).await.unwrap();
        sqlx::query(
            r"CREATE TABLE IF NOT EXISTS entities (
                entity_id BIGINT UNSIGNED PRIMARY KEY,
                chunk_id BIGINT UNSIGNED,
                world_id VARCHAR(50) NOT NULL,
                FOREIGN KEY(chunk_id) 
                    REFERENCES chunks(chunk_id),
                FOREIGN KEY(world_id)
                    REFERENCES worlds(world_id)
                )",
        ).execute(&self.pool).await.unwrap();
        sqlx::query(
            r"CREATE TABLE IF NOT EXISTS components (
                component_id BIGINT UNSIGNED PRIMARY KEY,
                type_id BIGINT UNSIGNED,
                dat TEXT,
                entity_id BIGINT UNSIGNED, 
                FOREIGN KEY(entity_id) 
                    REFERENCES entities(entity_id))",
        ).execute(&self.pool).await.unwrap();
        
    }
    pub async fn create_user(&self, username: &str, password: &str, is_admin : bool) -> bool {
        let pass = bcrypt::hash_with_result(password, 6).expect("Could not hash password");
        if !self.user_exists(username).await {
            sqlx::query(
                "INSERT INTO users (user_name, password_hash, admin) VALUES (?,?, ?)"
            ).bind(username).bind(pass.to_string()).bind(is_admin).execute(&self.pool).await.unwrap();
            return true;
        }
        false
    }
    pub async fn user_exists(&self, username: &str) -> bool {
        let x = sqlx::query(
            "SELECT admin FROM users WHERE user_name = ?"
        ).bind(username).fetch_one(&self.pool).await;
        let res = !x.is_err();
        res
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
    pub async  fn generate_session(&self, username: &str, password: &str) -> Option<String> {
        let mut user : Option<(String, bool)> = None;
        let row = sqlx::query(
                "SELECT password_hash, admin FROM users WHERE user_name = ?"
            ).bind(username).fetch_optional(&self.pool).await.unwrap();
        match row {
            Some(r) => {
                let u : &str = r.try_get("password_hash").unwrap();
                let admin : bool = r.try_get("admin").unwrap();
                user = Some((u.to_owned(),admin));
                println!("get");
            },
            None =>{
                println!("got");
            },
        }
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
    async fn listen_thread(listener: TcpListener, sv: Arc<RwLock<Self>>) {
        let lk = sv.read().unwrap();
        let key = lk.key.clone();
        drop(lk);
        loop {
            for (mut conn, addr) in listener.accept() {
                //spawn a worker thread
                let svnew = sv.clone();
                let key  =key.clone();
                task::spawn(async move {
                    let mut buf = &mut [0; 1024];
                    conn.read(buf);
                    match std::str::from_utf8(buf) {
                        Ok(buf) => {
                            match serde_json::from_str::<Value>(buf.trim_end_matches(char::from(0)))
                            {
                                Ok(buf) => match ServerRequest::new(buf,key.as_str()) {
                                    Ok((rx, req)) => {
                                        let t = std::thread::spawn(move ||  { task::block_on(async move {Self::worker_thread(req, svnew).await })}); 
                                        drop(t);
                                        match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                                            Ok(dat) => {
                                                conn.write(
                                                    serde_json::to_string(&dat.dat)
                                                        .unwrap()
                                                        .as_bytes(),
                                                );
                                            }
                                            Err(e) => {
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
        let g = game::Game::new("./raws", self.pool.clone(), world_name.to_owned());
        let gmrwlock = Arc::new(RwLock::new(g));
        let gmrwlock2 = gmrwlock.clone();
        game::Game::start_game(gmrwlock);
        self.game.insert(String::from(world_name), gmrwlock2);
    }
    async fn worker_thread(request: ServerRequest, sv: Arc<RwLock<Self>>) {
        match &request.dat {
            ServerRequestType::CreateGame { world_name } => {
                if request.is_admin() {
                    let mut guard = sv.write().unwrap();
                    guard.create_world(&world_name);
                    request.handle(ServerResponse::new(ServerResponseType::Ok {}));
                } else {
                    request.handle(ServerResponse::new(ServerResponseType::PermissionDenied {  }));
                }
            }
            ServerRequestType::Login { user, password } => {
                let guard = sv.read().unwrap();
                let x= guard.generate_session(&user, &password).await;
                match x {
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
    pub async fn new(args: &args::Args) -> Server {
        let (tx, rx) = crossbeam_channel::unbounded::<ServerRequest>();

        let key = args.secret.clone();
        let dburl = format!(
            "mysql://{}:{}@{}/{}",
            args.database_user, args.database_pass, args.database_host, args.database_name
        );
        println!("Connecting to database at {}", args.database_host);
        let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&dburl).await.expect("Could not get db conn");
        println!("Connection establised");
        Self {
            dburl: dburl,
            listen_url: format!("{}:{}", args.ip, args.port),
            pool: pool,
            game: HashMap::new(),
            key: key,
        }
    }
    pub async fn run_game(self) {
        self.initialize_database().await;
        if !self.user_exists("admin").await {
            println!("Creating user admin with default password \"password\"");
            self.create_user("admin", "password", true).await; 
        }
        let listener = TcpListener::bind(&self.listen_url).expect("Could not bind to ip/port");
        //create server arc
        let sv = Arc::new(RwLock::new(self));
        //create server listen thread
        Self::listen_thread(listener, sv.clone()).await;
    }
}

fn decode_valid_requests(stream: &TcpStream) -> Option<ServerRequest> {
    None
}
