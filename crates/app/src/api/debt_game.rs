use leptos::{prelude::ServerFnError, server};

#[server]
pub async fn get_random_background() -> Result<String, ServerFnError> {
    use rand::{prelude::IndexedRandom, rng};
    use std::{fs, path::Path};

    let dir_path = Path::new("data/images/debt_game");
    let default_bg = "/data/images/debt_background.jpg".to_string();

    if !dir_path.exists() || !dir_path.is_dir() {
        return Ok(default_bg);
    }

    let entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(_) => return Ok(default_bg),
    };

    let image_paths: Vec<String> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.is_file() {
                let extension = path.extension()?.to_str()?.to_lowercase();
                if ["jpg", "jpeg", "png", "gif", "webp"].contains(&extension.as_str()) {
                    if let Some(file_name) = path.file_name() {
                        if let Some(name_str) = file_name.to_str() {
                            return Some(format!("/data/images/debt_game/{}", name_str));
                        }
                    }
                }
            }
            None
        })
        .collect();

    if image_paths.is_empty() {
        return Ok(default_bg);
    }

    let mut rng = rng();
    if let Some(random_image) = image_paths.choose(&mut rng) {
        Ok(random_image.clone())
    } else {
        Ok(default_bg)
    }
}
