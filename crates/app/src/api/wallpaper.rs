use leptos::prelude::*;

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct WallpaperRecord {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub set_by: String,
    pub created_at: String,
}

#[server]
pub async fn get_current_wallpaper_filename() -> Result<Option<String>, ServerFnError> {
    use remote_bot_shared::state::AppState;
    use std::sync::Arc;
    let app_state = use_context::<Arc<AppState>>().expect("Missing AppState");
    let current = app_state.current_wallpaper_filename.lock().await;
    Ok(current.clone())
}

#[server]
pub async fn get_wallpapers(
    page: u32,
    per_page: u32,
) -> Result<Vec<WallpaperRecord>, ServerFnError<String>> {
    use remote_bot_shared::state::AppState;
    use sqlx::SqlitePool;
    use std::sync::Arc;

    let state: Arc<AppState> = use_context().expect("Missing AppState");
    let pool: SqlitePool = state.pool.clone();

    let limit = per_page as i64;
    let offset = (page.saturating_sub(1) * per_page) as i64;

    let recs = sqlx::query_as!(
        WallpaperRecord,
        r#"
        SELECT id, name, path, set_by, created_at as "created_at!: String"
        FROM wallpapers
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
        limit,
        offset
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(recs)
}
