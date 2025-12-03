use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cache::ShouldCache;

#[derive(Serialize, Deserialize, Debug)]
pub enum GetAppPathError {
    ExecutionFailed(String),
    ParsingFailed(String),
}

fn _get_app_path(app: &str) -> Result<String, GetAppPathError> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "tell application \"System Events\" to POSIX path of (file of process \"{}\" as alias)",
            app
        ))
        .stdout(std::process::Stdio::piped())
        .output()
        .map_err(|e| GetAppPathError::ExecutionFailed(e.to_string()))?;

    let output = String::from_utf8(output.stdout)
        .map_err(|e| GetAppPathError::ParsingFailed(e.to_string()))?;

    Ok(output)
}

pub fn get_app_path(app: &str) -> (ShouldCache, Result<String, GetAppPathError>) {
    match _get_app_path(app) {
        Ok(path) => (ShouldCache::Yes, Ok(path)),
        Err(e) => (ShouldCache::No, Err(e)),
    }
}
