use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub current_wallpaper_filename: Arc<Mutex<Option<String>>>,
}
