use actix_files::Files;
use actix_web::{App, HttpServer, middleware::Compress};
use config::Config as AppConfig;
use leptos::{
    config::get_config_from_env,
    hydration::{AutoReload, HydrationScripts},
    prelude::{ElementChild, GlobalAttributes, provide_context},
    view,
};
use leptos_actix::{LeptosRoutes, generate_route_list};
use leptos_dom::log;
use leptos_meta::MetaTags;
use remote_bot_discord::{configuration::Config, run_bot};
use remote_bot_shared::state::AppState as LeptosAppState;
use sqlx::SqlitePool;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    _ = dotenvy::dotenv();

    let conf = get_config_from_env().unwrap();
    let addr = conf.leptos_options.site_addr;

    let pool = SqlitePool::connect("sqlite:wallpapers.db").await.unwrap();
    sqlx::migrate!("./../../migrations")
        .run(&pool)
        .await
        .unwrap();

    let leptos_app_state = Arc::new(LeptosAppState { pool });

    let settings: Config = AppConfig::builder()
        .add_source(config::File::with_name("settings"))
        .add_source(config::Environment::with_prefix("REMOTE_BOT"))
        .build()
        .expect("Expected a settings file!")
        .try_deserialize::<Config>()
        .expect("Failed to deserialize settings");

    tokio::spawn(run_bot(leptos_app_state.clone(), settings.clone()));

    log!("Server starting at {}", addr);

    let conf = get_config_from_env().unwrap().clone();
    let leptos_options = conf.leptos_options.clone();
    let site_root = leptos_options.site_root.clone();
    let bind_addr = leptos_options.site_addr.clone();

    HttpServer::new(move || {
        let leptos_options = leptos_options.clone();
        let leptos_app_state_clone = leptos_app_state.clone();

        App::new()
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
            .service(Files::new("/", &*site_root))
            .service(Files::new("/data", "data"))
            .wrap(Compress::default())
    })
    .bind(&bind_addr)?
    .run()
    .await
}
