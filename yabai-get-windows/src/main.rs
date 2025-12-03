mod app_paths;
mod cache;
mod error;

use crate::app_paths::get_app_path;
use anyhow::{Context, Result};
use log::error;
use serde::Deserialize;
use std::collections::BTreeMap as Map;
use std::fmt::format;
use std::io;
use std::process::Output;
use which::which;

extern crate alfred;

#[derive(Debug, Deserialize)]
struct YabaiWindow {
    id: u32,
    app: String,
    title: String,
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

    let mut get_app_path = cache::Cache::new(get_app_path, "/tmp/app_paths.db");

    let windows_items: Vec<alfred::Item> = windows
        .iter()
        .filter_map(|window| {
            get_app_path
                .call(&window.app)
                .inspect_err(|err| error!("Failed to get app path for {}: {:?}", window.app, err))
                .map(|path| (window, path))
                .ok()
        })
        .map(|(window, path)| {
            let title = format!("{} - {}", window.app, window.title);
            let arg = format!("{}", window.id);
            alfred::ItemBuilder::new(title.clone())
                .uid(title)
                .subtitle(&window.app)
                .icon_file(path)
                .arg(arg)
                .into_item()
        })
        .collect();

    alfred::json::write_items(io::stdout(), &windows_items)
        .context("Failed to write alfred items to stdout")?;

    Ok(())
}
