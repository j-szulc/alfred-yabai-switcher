mod app_paths;
mod cache;

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, error, info};
use serde::Deserialize;
use std::{borrow::Cow, io, path::PathBuf};
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

macro_rules! timeit_ {
    ($expr:expr, $label:literal) => {{
        let start = std::time::Instant::now();
        let result = $expr;
        let end = std::time::Instant::now();
        debug!(
            "Time taken by {:?}: {}",
            $label,
            humantime::format_duration(end.duration_since(start))
        );
        result
    }};
}

macro_rules! timeit {
    ($expr:expr, $label:literal) => {{
        if log::log_enabled!(log::Level::Debug) {
            timeit_!($expr, $label)
        } else {
            $expr
        }
    }};
}

macro_rules! error_multiline {
    ($header:expr, $message:expr) => {
        let message = $message;
        let message_trimmed = message.trim_ascii();
        if message_trimmed.is_empty() {
            error!("{} empty", $header);
        } else {
            error!("{}", $header);
            for line in message_trimmed.lines() {
                error!("    {}", line.trim());
            }
        }
    };
}

#[derive(Parser)]
struct Args {
    #[arg(long = "bin", help = "Path to the yabai binary")]
    yabai_bin: Option<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let yabai_bin = match args.yabai_bin {
        Some(path) => path,
        None => which("yabai").context("Executable 'yabai' not found in PATH")?,
    };

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

    let windows: Vec<YabaiWindow> = timeit!(
        serde_json::from_slice(&output.stdout)
            .inspect_err(|_| {
                error_multiline!("Yabai stdout:", String::from_utf8_lossy(&output.stdout));
                error_multiline!("Yabai stderr:", String::from_utf8_lossy(&output.stderr));
                info!("Hint: Try running `yabai --stop-service ; yabai --start-service`");
            })
            .context("Failed to parse yabai output as JSON")?,
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
                        error!("Failed to get app path for {}: {err:?}", window.app);
                    })
                    .ok()
                    .map(|path| (window, (*path).to_string()))
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
