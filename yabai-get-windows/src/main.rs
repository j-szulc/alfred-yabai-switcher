use std::collections::BTreeMap as Map;
use std::fmt::format;
use std::process::Output;
use serde::{Deserialize, Serialize};
extern crate alfred;
use std::io;

#[derive(Debug, Deserialize)]
struct YabaiWindow{
    id: u32,
    pid: u32,
    app: String,
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppLocationPathCache {
    entries: Map<String, String>,
}

fn get_app_path(app: &str) -> std::io::Result<String> {
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "tell application \"System Events\" to POSIX path of (file of process \"{}\" as alias)",
            app
        ))
        .stdout(std::process::Stdio::piped())
        .output()
        .and_then(|output: Output| {
            let output = String::from_utf8(output.stdout).unwrap();
            Ok(output.trim().to_string())
        })
}

fn get_app_path_cached(cache: &mut AppLocationPathCache, app: &str) -> std::io::Result<String> {
    if let Some(path) = cache.entries.get(app) {
        return Ok(path.to_string());
    }

    let path = get_app_path(app)?;
    cache.entries.insert(app.to_string(), path.clone());
    Ok(path)
}

fn load_cache() -> std::io::Result<AppLocationPathCache> {
    let cache_path = std::path::Path::new("/tmp/yabai-app-location-path-cache.json");
    if !cache_path.exists() {
        return Ok(AppLocationPathCache {
            entries: Map::new(),
        });
    }

    let cache_file = std::fs::File::open(cache_path)?;
    let cache: AppLocationPathCache = serde_json::from_reader(cache_file)?;
    Ok(cache)
}

fn save_cache(cache: &AppLocationPathCache) -> std::io::Result<()> {
    let cache_path = std::path::Path::new("/tmp/yabai-app-location-path-cache.json");
    let cache_file = std::fs::File::create(cache_path)?;
    serde_json::to_writer(cache_file, cache)?;
    Ok(())
}

fn main() {

    let mut cache = load_cache().unwrap();

    let output = std::process::Command::new("yabai")
        .arg("-m")
        .arg("query")
        .arg("--windows")
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8(output.stdout).unwrap();
    let windows: Vec<YabaiWindow> = serde_json::from_str(&output).unwrap();

    let windows_items: Vec<alfred::Item> = windows
        .iter()
        .map(|window| {
            let path = get_app_path_cached(&mut cache, &window.app).unwrap();
            let title = format!("{} - {}", window.app, window.title);
            let arg = format!("{}", window.id);
            alfred::ItemBuilder::new(title.clone())
                .uid(title.clone())
                .subtitle(&window.app)
                .icon_file(path.clone())
                .arg(arg.clone())
                .into_item()
        })
        .collect();

    alfred::json::write_items(io::stdout(), &windows_items).unwrap();

    save_cache(&cache).unwrap();
}
