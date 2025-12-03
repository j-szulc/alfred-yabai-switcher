use anyhow::{Context, Result};
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;
use std::fmt::format;
use std::io;
use std::process::Output;
use which::which;

extern crate alfred;

#[derive(Debug, Deserialize)]
struct YabaiWindow {
    id: u32,
    #[allow(dead_code)]
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

fn main() -> Result<()> {
    env_logger::init();

    let yabai_bin = which("yabai").context("Executable 'yabai' not found in PATH")?;

    let output = std::process::Command::new(yabai_bin)
        .arg("-m")
        .arg("query")
        .arg("--windows")
        .output()
        .context("Failed to execute yabai")?;

    let output = String::from_utf8(output.stdout)
        .context("Failed to parse yabai output as UTF-8")
        .inspect_err(|err| {
            error!("Failed to parse yabai output as UTF-8: {}", err);
        })?;

    let windows: Vec<YabaiWindow> = serde_json::from_str(&output)
        .context("Failed to parse yabai output as JSON")
        .inspect_err(|err| {
            error!("Failed to parse yabai output as JSON: {}", err);
        })?;

    let windows_items: Vec<alfred::Item> = windows
        .iter()
        .filter_map(|window| {
            get_app_path(&window.app)
                .inspect_err(|err| error!("Failed to get app path for {}: {}", window.app, err))
                .map(|path| (window, path))
                .ok()
        })
        .map(|(window, path)| {
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

    alfred::json::write_items(io::stdout(), &windows_items)
        .context("Failed to write alfred items to stdout")?;

    Ok(())
}
