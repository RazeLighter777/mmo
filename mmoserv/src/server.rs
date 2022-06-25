use crate::args;
use crate::game;
use crate::server_request::ServerClaims;
use crate::server_request::ServerRequest;
use crate::sql_loaders;
use futures::task::noop_waker;
use mmolib::server_request_type::ServerRequestType;
use mmolib::server_response_type::ServerResponseType;
use tokio::runtime::Handle;
use tokio::sync::RwLock;
//use tokio::prelude::*;
use bcrypt::bcrypt;
use crossbeam_channel::internal::SelectHandle;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use jsonwebtoken::TokenData;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use log::{info, warn};
use tokio::task;

use futures::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::mysql::MySqlRow;
use sqlx::pool::PoolConnection;
use sqlx::Acquire;
use sqlx::MySql;
use sqlx::Pool;
use sqlx::Row;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Context;
use std::thread::park;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub struct Server {
    dburl: String,
    pool: Pool<MySql>,
    game: HashMap<String, Arc<RwLock<game::Game>>>,
    key: String,
    listen_url: String,
    open_streams: Vec<Arc<RwLock<WebSocketStream<TcpStream>>>>,
}

impl Server {
    pub async fn create_user(&self, username: &str, password: &str, is_admin: bool) -> bool {
        let pass = bcrypt::hash_with_result(password, 6).expect("Could not hash password");
        if !self.user_exists(username).await {
            sqlx::query("INSERT INTO users (user_name, password_hash, admin) VALUES (?,?, ?)")
                .bind(username)
                .bind(pass.to_string())
                .bind(is_admin)
                .execute(&self.pool)
                .await
                .unwrap();
            return true;
        }
        false
    }
    pub async fn user_exists(&self, username: &str) -> bool {
        let x = sqlx::query("SELECT admin FROM users WHERE user_name = ?")
            .bind(username)
            .fetch_one(&self.pool)
            .await;
        let res = !x.is_err();
        res
    }
    pub fn get_claims(&self, session: &str) -> Option<ServerClaims> {
        let token = decode::<ServerClaims>(
            session,
            &DecodingKey::from_secret(self.key.as_ref()),
            &Validation::default(),
        );
        match token {
            Ok(tok) => Some(tok.claims),
            Err(e) => {
                println!("{}", e);
                None
            }
        }
    }
    pub async fn generate_session(&self, username: &str, password: &str) -> Option<String> {
        let mut user: Option<(String, bool)> = None;
        let row = sqlx::query("SELECT password_hash, admin FROM users WHERE user_name = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .unwrap();
        match row {
            Some(r) => {
                let u: &str = r.try_get("password_hash").unwrap();
                let admin: bool = r.try_get("admin").unwrap();
                user = Some((u.to_owned(), admin));
                println!("get");
            }
            None => {
                println!("got");
            }
        }
        match user {
            Some((u, is_admin)) => match bcrypt::verify(password, &u) {
                Ok(b) => {
                    if b {
                        let claims = ServerClaims {
                            user_name: String::from(username),
                            is_admin: is_admin,
                            exp: 10000000000,
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
        let lk = sv.read().await;
        let key = lk.key.clone();
        drop(lk);
        loop {
            for (mut conn, addr) in listener.accept().await {
                //spawn a worker thread
                let svnew = sv.clone();
                let key = key.clone();
                task::spawn(async move {
                    let (wsw, mut wsr) = tokio_tungstenite::accept_async(conn)
                        .await
                        .expect("Could not listen on the websocket connection")
                        .split();
                    //there can be multiple connection senders, but only one reader. That's why ws write (wsw) is in an arc.
                    let mut wsw = Arc::new(RwLock::new(wsw));
                    loop {
                        //loop until connection is terminated
                        match wsr.next().await {
                            Some(msg) => match msg {
                                Ok(msg) => match msg.to_text() {
                                    Ok(txt) => match serde_json::from_str(txt) {
                                        Ok(json_value) => {
                                            match ServerRequest::new(
                                                json_value,
                                                &key.clone(),
                                                wsw.clone(),
                                            ) {
                                                Ok(request) => {
                                                    Self::worker_thread(request, svnew.clone())
                                                        .await;
                                                }
                                                Err(_) => {
                                                    println!("Was valid json but not valid server request");
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            println!("Was not valid json");
                                        }
                                    },
                                    Err(_) => {
                                        println!("Websocket message was not text");
                                    }
                                },
                                Err(_) => {
                                    println!("Connection closed");
                                    break;
                                }
                            },
                            None => {
                                println!("Could not await next message from websocket reader");
                                break;
                            }
                        }
                    }
                });
            }
        }
    }
    pub async fn create_world(&mut self, world_name: &str) -> bool {
        if sql_loaders::create_world(self.pool.clone(), world_name).await {
            let g = game::Game::new(
                "C:\\Users\\justin\\Code\\mmo\\raws",
                self.pool.clone(),
                world_name.to_owned(),
            );
            //insert the world into the database
            let gmrwlock = Arc::new(RwLock::new(g));
            let gmrwlock2 = gmrwlock.clone();
            game::Game::start_game(gmrwlock).await;
            self.game.insert(String::from(world_name), gmrwlock2);
            true
        } else {
            false
        }
    }
    async fn worker_thread(request: ServerRequest, sv: Arc<RwLock<Self>>) {
        match &request.get_dat() {
            ServerRequestType::CreateGame { world_name } => {
                if request.is_admin() {
                    let mut guard = sv.write().await;
                    if guard.create_world(&world_name).await {
                        request.handle(&ServerResponseType::Ok {}).await;
                    } else {
                        request
                            .handle(&ServerResponseType::Error {
                                message: "World already exists",
                            })
                            .await;
                    }
                } else {
                    request
                        .handle(&ServerResponseType::PermissionDenied {})
                        .await;
                }
            }
            ServerRequestType::Login { user, password } => {
                let guard = sv.read().await;
                let x = guard.generate_session(&user, &password).await;
                match x {
                    Some(token) => {
                        request
                            .handle(&ServerResponseType::AuthSuccess {
                                session_token: token,
                            })
                            .await;
                    }
                    None => {
                        request.handle(&ServerResponseType::AuthFailure {}).await;
                    }
                }
            }
            other => match request.get_world().to_owned() {
                Some(world_name) => {
                    let guard = sv.read().await;
                    match guard.game.get(&world_name) {
                        Some(gm) => match request.get_user() {
                            Some(user) => {
                                let gmc = gm.clone();
                                game::Game::handle(gmc, request).await;
                            }
                            None => {
                                request.handle(&ServerResponseType::AuthFailure {}).await;
                                println!("User must be logged in to join {}", &world_name);
                            }
                        },
                        None => {
                            request.handle(&ServerResponseType::AuthFailure {}).await;
                            println!("World {} doesn't exist yet", &world_name);
                        }
                    }
                }
                None => {}
            },
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
            .connect(&dburl)
            .await
            .expect("Could not get db conn");
        println!("Connection establised");
        Self {
            dburl: dburl,
            listen_url: format!("{}:{}", args.ip, args.port),
            pool: pool,
            game: HashMap::new(),
            key: key,
            open_streams: Vec::new(),
        }
    }
    pub async fn run_game(self) {
        sql_loaders::initialize_database(self.pool.clone()).await;
        if !self.user_exists("admin").await {
            println!("Creating user admin with default password \"password\"");
            self.create_user("admin", "password", true).await;
        }
        let listener = TcpListener::bind(&self.listen_url)
            .await
            .expect("Could not bind to ip/port");
        //create server arc
        let sv = Arc::new(RwLock::new(self));
        //create server listen thread
        Self::listen_thread(listener, sv.clone()).await;
    }
}

fn decode_valid_requests(stream: &TcpStream) -> Option<ServerRequest> {
    None
}
