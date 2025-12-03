mod app_paths;
mod cache;
mod utils;

use anyhow::{Context, Result};
use log::error;
use serde::Deserialize;
use std::{
    borrow::Cow,
    io,
    path::{Path, PathBuf},
};
use which::which;

use crate::{app_paths::get_app_path, cache::Cache};

extern crate alfred;

const CACHE_FILE: &str = "/tmp/app_paths.json";

#[derive(Debug, Deserialize)]
struct YabaiWindow {
    id: u32,
    app: String,
    title: String,
}

macro_rules! timeit {
    ($expr:expr, $label:literal) => {{
        let start = std::time::Instant::now();
        let result = $expr;
        let end = std::time::Instant::now();
        eprintln!(
            "Time taken for {}: {}",
            $label,
            humantime::format_duration(end.duration_since(start))
        );
        result
    }};
}

fn _main() -> Result<()> {
    let yabai_bin = which("yabai").context("Executable 'yabai' not found in PATH")?;

    // TODO: async and timeout
    let output = timeit!(
        std::process::Command::new(yabai_bin)
            .arg("-m")
            .arg("query")
            .arg("--windows")
            .stdin(std::process::Stdio::null())
            .output()
            .context("Failed to execute yabai")?,
        "yabai query"
    );

    let output = timeit!(
        String::from_utf8(output.stdout)
            .context("Failed to parse yabai output as UTF-8")
            .inspect_err(|err| {
                error!("Failed to parse yabai output as UTF-8: {}", err);
            })?,
        "parse yabai output"
    );

    let windows: Vec<YabaiWindow> = timeit!(
        serde_json::from_str(&output)
            .context("Failed to parse yabai output as JSON")
            .inspect_err(|err| {
                error!("Failed to parse yabai output as JSON: {}", err);
            })?,
        "parse yabai output as JSON"
    );

    let mut cache: Cache<Cow<'_, str>, String> = Cache::new(&PathBuf::from(CACHE_FILE))?;

    let windows_items: Vec<alfred::Item> = timeit!(
        windows
            .iter()
            .filter_map(|window| {
                cache
                    .get_or_insert_with(Cow::Borrowed(&window.app), || get_app_path(&window.app))
                    .inspect_err(|err| {
                        error!("Failed to get app path for {}: {:?}", window.app, err)
                    })
                    .ok()
                    .map(|path| (window, path.to_string()))
                // .inspect(|path| eprintln!("Got app path: {}", path))
            })
            .map(|(window, path)| {
                let title = format!("{} - {}", window.app.trim(), window.title.trim());
                let arg = format!("{}", window.id);
                alfred::ItemBuilder::new(title.clone())
                    .uid(title)
                    .subtitle(window.app.trim())
                    .icon_file(path)
                    .arg(arg)
                    .into_item()
            })
            .collect(),
        "get alfred items"
    );

    alfred::json::write_items(io::stdout(), &windows_items)
        .context("Failed to write alfred items to stdout")?;

    Ok(())
}

fn main() {
    timeit!(
        {
            env_logger::init();

            if let Err(e) = _main() {
                error!("{}", e);
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        "main"
    )
}
