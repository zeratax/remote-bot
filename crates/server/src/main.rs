use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_files::Files;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, middleware::Compress};
use actix_web_actors::ws;
use config::Config as AppConfig;
use leptos::{
    config::get_config_from_env,
    prelude::{ElementChild, GlobalAttributes, provide_context},
    view, hydration::{AutoReload, HydrationScripts},
};
use leptos_actix::{LeptosRoutes, generate_route_list};
use leptos_dom::log;
use leptos_meta::MetaTags;
use remote_bot_discord::{configuration::Config, run_bot};
use remote_bot_shared::state::AppState as LeptosAppState;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

// Constants for WebSocket game server
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(120);
const GAME_CLEANUP_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes
const GAME_INACTIVITY_TIMEOUT: Duration = Duration::from_secs(86400); // 24 hours

// Message types
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "join")]
    Join { game_id: String },
    #[serde(rename = "leave")]
    Leave {},
    #[serde(rename = "gameAction")]
    GameAction { game_id: String, action: serde_json::Value },
    #[serde(rename = "gameState")]
    GameState { game_id: String, state: serde_json::Value },
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
enum ServerMessage {
    #[serde(rename = "connection")]
    Connection {
        client_id: String,
        message: String,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
    #[serde(rename = "joinedGame")]
    JoinedGame {
        game_id: String,
        current_state: Option<serde_json::Value>,
    },
    #[serde(rename = "playerJoined")]
    PlayerJoined {
        game_id: String,
        client_id: String,
    },
    #[serde(rename = "playerLeft")]
    PlayerLeft {
        game_id: String,
        client_id: String,
    },
    #[serde(rename = "gameAction")]
    GameAction {
        game_id: String,
        client_id: String,
        action: serde_json::Value,
    },
    #[serde(rename = "gameState")]
    GameState {
        game_id: String,
        state: serde_json::Value,
    },
    #[serde(rename = "gameRemoved")]
    GameRemoved {
        game_id: String,
        reason: String,
    },
}

// Game data structures
struct Game {
    clients: HashSet<String>,
    state: Option<serde_json::Value>,
    last_update: Instant,
}

// Game server state
struct GameServerState {
    active_games: Mutex<HashMap<String, Game>>,
    active_connections: Mutex<HashMap<String, GameConnection>>,
}

struct GameConnection {
    addr: actix::Addr<GameSession>,
    game_id: Option<String>,
}

// WebSocket session
struct GameSession {
    id: String,
    hb: Instant,
    app: Arc<GameServerState>,
}

impl GameSession {
    fn new(id: String, app: Arc<GameServerState>) -> Self {
        Self {
            id,
            hb: Instant::now(),
            app,
        }
    }

    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Client timeout: {}", act.id);
                
                // Handle client disconnection
                act.handle_leave_game();
                
                // Stop the actor
                ctx.stop();
                return;
            }
            
            ctx.ping(b"");
        });
    }

    fn handle_join_game(&self, game_id: String) {
        let mut games = self.app.active_games.lock().unwrap();
        let mut connections = self.app.active_connections.lock().unwrap();
        
        // Leave any current game
        self.handle_leave_game();
        
        // Join or create the game
        let game = games.entry(game_id.clone()).or_insert_with(|| {
            Game {
                clients: HashSet::new(),
                state: None,
                last_update: Instant::now(),
            }
        });
        
        game.clients.insert(self.id.clone());
        
        if let Some(connection) = connections.get_mut(&self.id) {
            connection.game_id = Some(game_id.clone());
        }
        
        println!("Client {} joined game {}", self.id, game_id);
        
        // Notify client they've joined
        if let Some(connection) = connections.get(&self.id) {
            let message = ServerMessage::JoinedGame {
                game_id: game_id.clone(),
                current_state: game.state.clone(),
            };
            
            let msg = serde_json::to_string(&message).unwrap();
            connection.addr.do_send(GameMessage(msg));
        }
        
        // Notify other clients in the game
        let message = ServerMessage::PlayerJoined {
            game_id: game_id.clone(),
            client_id: self.id.clone(),
        };
        
        self.broadcast_to_game(&game_id, &message, Some(&self.id));
    }

    fn handle_leave_game(&self) {
        let mut games = self.app.active_games.lock().unwrap();
        let mut connections = self.app.active_connections.lock().unwrap();
        
        let connection = if let Some(conn) = connections.get_mut(&self.id) {
            conn
        } else {
            return;
        };
        
        if let Some(game_id) = &connection.game_id {
            if let Some(game) = games.get_mut(game_id) {
                game.clients.remove(&self.id);
                
                // If no clients left, remove the game
                if game.clients.is_empty() {
                    games.remove(game_id);
                    println!("Game {} removed (no players)", game_id);
                } else {
                    // Notify other clients in the game
                    let message = ServerMessage::PlayerLeft {
                        game_id: game_id.clone(),
                        client_id: self.id.clone(),
                    };
                    
                    self.broadcast_to_game(game_id, &message, None);
                }
            }
            
            println!("Client {} left game {}", self.id, game_id);
            connection.game_id = None;
        }
    }

    fn handle_game_action(&self, game_id: String, action: serde_json::Value) {
        let games = self.app.active_games.lock().unwrap();
        let connections = self.app.active_connections.lock().unwrap();
        
        // Verify client is in the game
        let connection = if let Some(conn) = connections.get(&self.id) {
            conn
        } else {
            return;
        };
        
        if connection.game_id.as_deref() != Some(&game_id) {
            return;
        }
        
        if !games.contains_key(&game_id) {
            return;
        }
        
        // Broadcast the action to all clients in the game
        let message = ServerMessage::GameAction {
            game_id: game_id.clone(),
            client_id: self.id.clone(),
            action,
        };
        
        self.broadcast_to_game(&game_id, &message, None);
        
        println!("Game action from {} in game {}", self.id, game_id);
    }

    fn handle_game_state(&self, game_id: String, state: serde_json::Value) {
        let mut games = self.app.active_games.lock().unwrap();
        let connections = self.app.active_connections.lock().unwrap();
        
        // Verify client is in the game
        let connection = if let Some(conn) = connections.get(&self.id) {
            conn
        } else {
            return;
        };
        
        if connection.game_id.as_deref() != Some(&game_id) {
            return;
        }
        
        let game = if let Some(g) = games.get_mut(&game_id) {
            g
        } else {
            return;
        };
        
        // Update the game state
        game.state = Some(state.clone());
        game.last_update = Instant::now();
        
        // Broadcast to all clients in the game
        let message = ServerMessage::GameState {
            game_id: game_id.clone(),
            state,
        };
        
        self.broadcast_to_game(&game_id, &message, Some(&self.id));
        
        println!("Game state updated in game {} by {}", game_id, self.id);
    }

    fn broadcast_to_game(&self, game_id: &str, message: &ServerMessage, exclude_client_id: Option<&str>) {
        let games = self.app.active_games.lock().unwrap();
        let connections = self.app.active_connections.lock().unwrap();
        
        let game = if let Some(g) = games.get(game_id) {
            g
        } else {
            return;
        };
        
        let message_str = serde_json::to_string(message).unwrap();
        
        for client_id in &game.clients {
            if let Some(exclude_id) = exclude_client_id {
                if client_id == exclude_id {
                    continue;
                }
            }
            
            if let Some(connection) = connections.get(client_id) {
                connection.addr.do_send(GameMessage(message_str.clone()));
            }
        }
    }
}

// Message wrapper for WebSocket text messages
struct GameMessage(String);

impl actix::Message for GameMessage {
    type Result = ();
}

impl actix::Handler<GameMessage> for GameSession {
    type Result = ();

    fn handle(&mut self, msg: GameMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Actor for GameSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);

        // Register the session with the app state
        let mut connections = self.app.active_connections.lock().unwrap();
        
        connections.insert(self.id.clone(), GameConnection {
            addr: ctx.address(),
            game_id: None,
        });
        
        println!("New connection established: {}", self.id);
        
        // Send welcome message
        let message = ServerMessage::Connection {
            client_id: self.id.clone(),
            message: "Connected to game sync server".to_string(),
        };
        
        let msg = serde_json::to_string(&message).unwrap();
        ctx.text(msg);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        println!("Connection closed: {}", self.id);
        
        // Handle client disconnection
        self.handle_leave_game();
        
        // Remove the session from the app state
        let mut connections = self.app.active_connections.lock().unwrap();
        connections.remove(&self.id);
        
        actix::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(message) => match message {
                        ClientMessage::Join { game_id } => {
                            self.handle_join_game(game_id);
                        }
                        ClientMessage::Leave {} => {
                            self.handle_leave_game();
                        }
                        ClientMessage::GameAction { game_id, action } => {
                            self.handle_game_action(game_id, action);
                        }
                        ClientMessage::GameState { game_id, state } => {
                            self.handle_game_state(game_id, state);
                        }
                    },
                    Err(e) => {
                        println!("Error parsing message from {}: {}", self.id, e);
                        let message = ServerMessage::Error {
                            message: "Invalid message format".to_string(),
                        };
                        
                        let msg = serde_json::to_string(&message).unwrap();
                        ctx.text(msg);
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    game_state: web::Data<Arc<GameServerState>>,
) -> Result<HttpResponse, Error> {
    let client_id = Uuid::new_v4().to_string();
    
    // Fix: Directly use the content without an extra clone
    ws::start(
        GameSession::new(client_id, game_state.get_ref().clone()),
        &req,
        stream,
    )
}

// Setup WebSocket game server cleanup task
fn setup_game_cleanup(game_state: Arc<GameServerState>) {
    actix_web::rt::spawn(async move {
        let mut interval = actix_web::rt::time::interval(GAME_CLEANUP_INTERVAL);
        
        loop {
            interval.tick().await;
            
            let mut games = game_state.active_games.lock().unwrap();
            let connections = game_state.active_connections.lock().unwrap();
            let now = Instant::now();
            let mut removed_count = 0;
            
            let stale_games: Vec<String> = games
                .iter()
                .filter(|(_, game)| now.duration_since(game.last_update) > GAME_INACTIVITY_TIMEOUT)
                .map(|(id, _)| id.clone())
                .collect();
            
            for game_id in stale_games {
                if let Some(game) = games.get(&game_id) {
                    // Notify all clients in the game
                    let message = ServerMessage::GameRemoved {
                        game_id: game_id.clone(),
                        reason: "inactivity".to_string(),
                    };
                    
                    let message_str = serde_json::to_string(&message).unwrap();
                    
                    for client_id in &game.clients {
                        if let Some(connection) = connections.get(client_id) {
                            connection.addr.do_send(GameMessage(message_str.clone()));
                        }
                    }
                }
                
                games.remove(&game_id);
                removed_count += 1;
            }
            
            // Reset game_id for clients who were in removed games
            if removed_count > 0 {
                drop(games); // Release lock on games
                let mut connections = game_state.active_connections.lock().unwrap();
                
                for (_, connection) in connections.iter_mut() {
                    if let Some(game_id) = &connection.game_id {
                        if !game_state.active_games.lock().unwrap().contains_key(game_id) {
                            connection.game_id = None;
                        }
                    }
                }
                
                println!("Cleaned up {} stale games", removed_count);
            }
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    _ = dotenvy::dotenv();
    
    // Get Leptos configuration
    let conf = get_config_from_env().unwrap();
    let addr = conf.leptos_options.site_addr;
    
    // Setup SQLite database connection
    let pool = SqlitePool::connect("sqlite:wallpapers.db").await.unwrap();
    sqlx::migrate!("./../../migrations")
        .run(&pool)
        .await
        .unwrap();
    
    // Get last wallpaper path from database
    let last_path: Option<String> =
        sqlx::query_scalar("SELECT path FROM wallpapers ORDER BY created_at DESC LIMIT 1")
            .fetch_optional(&pool)
            .await
            .unwrap();
    
    // Create Leptos app state
    let leptos_app_state = Arc::new(LeptosAppState {
        pool,
        current_wallpaper_filename: Arc::new(TokioMutex::new(last_path)),
    });
    
    // Load configuration for Discord bot
    let settings: Config = AppConfig::builder()
        .add_source(config::File::with_name("settings"))
        .add_source(config::Environment::with_prefix("REMOTE_BOT"))
        .build()
        .expect("Expected a settings file!")
        .try_deserialize::<Config>()
        .expect("Failed to deserialize settings");
    
    // Run Discord bot
    tokio::spawn(run_bot(leptos_app_state.clone(), settings.clone()));
    
    // Create game server state
    let game_state = Arc::new(GameServerState {
        active_games: Mutex::new(HashMap::new()),
        active_connections: Mutex::new(HashMap::new()),
    });
    
    // Setup game cleanup task
    setup_game_cleanup(game_state.clone());
    
    // Log server information
    log!("Server starting at {}", addr);
    
    // Get Leptos configuration for HTTP server
    let conf = get_config_from_env().unwrap().clone();
    let leptos_options = conf.leptos_options.clone();
    let site_root = leptos_options.site_root.clone();
    let bind_addr = leptos_options.site_addr.clone();
    
    // Start HTTP server with both Leptos and WebSocket routes
    HttpServer::new(move || {
        let leptos_options = leptos_options.clone();
        let leptos_app_state_clone = leptos_app_state.clone();
        let game_state_clone = game_state.clone();
        
        App::new()
            // Leptos routes with context
            .leptos_routes_with_context(
                generate_route_list(remote_bot_app::App), 
                {
                    let state = leptos_app_state_clone.clone();
                    move || provide_context(state.clone())
                }, 
                move || {
                    let leptos_options = leptos_options.clone();
                    view! {
                        <!DOCTYPE html>
                        <html lang="en">
                            <head>
                                <meta charset="utf-8"/>
                                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                <AutoReload options=leptos_options.clone()/>
                                <HydrationScripts options=leptos_options.clone()/>
                                <MetaTags/>
                            </head>
                            <body>
                                <remote_bot_app::App/>
                            </body>
                        </html>
                    }
                }
            )
            // WebSocket route for game server
            .app_data(web::Data::new(game_state_clone.clone()))
            .route("/ws", web::get().to(websocket_route))
            // Static files
            .service(Files::new("/", &*site_root))
            // Compression middleware
            .wrap(Compress::default())
    })
    .bind(&bind_addr)?
    .run()
    .await
}
