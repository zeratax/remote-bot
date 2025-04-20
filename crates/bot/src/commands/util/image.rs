use std::path::{Path, PathBuf};
use tokio::fs;
use url::Url;

use crate::commands::util::email::send_email;
use crate::configuration::Config;
use remote_bot_shared::state::AppState;

use uuid::Uuid;

pub async fn download_and_save_image(url: &str, save_dir: &PathBuf) -> Result<PathBuf, String> {
    let parsed_url = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
    let original_filename = parsed_url
        .path_segments()
        .and_then(|segments| segments.last())
        .ok_or_else(|| "Failed to extract filename from URL".to_string())?;

    let ext = Path::new(original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");

    let stem = Path::new(original_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let filename = format!("{}_{}.{}", stem, Uuid::now_v7(), ext);
    let save_path = save_dir.join(&filename);

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;

    let content = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image content: {}", e))?;

    if !fs::try_exists(save_dir).await.unwrap_or(false) {
        fs::create_dir_all(save_dir)
            .await
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(&save_path, &content)
        .await
        .map_err(|e| format!("Failed to write image to file: {}", e))?;

    Ok(PathBuf::from(filename))
}

async fn handle_set_wallpaper_from_path(
    path: PathBuf,
    set_by: &str,
    app_state: &AppState,
    config: &Config,
) -> Result<(), String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?
        .to_string();

    let path_str = path.to_string_lossy().to_string();

    sqlx::query!(
        "INSERT INTO wallpapers (name, path, set_by) VALUES (?, ?, ?)",
        name,
        path_str,
        set_by
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| format!("Failed to insert wallpaper: {}", e))?;

    {
        let mut current = app_state.current_wallpaper_filename.lock().await;
        *current = Some(path_str);
    }

    let body = format!("Wallpaper set by {}: {}", set_by, name);
    send_email(config, "Wallpaper Update", &body, "wallpaper").await?;

    Ok(())
}

pub async fn handle_set_wallpaper_from_url(
    url: &str,
    save_dir: &PathBuf,
    set_by: &str,
    app_state: &AppState,
    config: &Config,
) -> Result<(), String> {
    let path = download_and_save_image(url, save_dir).await?;
    handle_set_wallpaper_from_path(path, set_by, app_state, config).await
}
