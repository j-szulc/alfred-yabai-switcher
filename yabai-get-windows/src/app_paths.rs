use anyhow::{Context, Result};
use log::error;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::hash_map::Entry::{self, Occupied, Vacant};
use std::fs;
use std::process::Command;
use std::{collections::HashMap, process::Stdio};

pub fn get_app_path(app: &str) -> Result<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "tell application \"System Events\" to POSIX path of (file of process \"{}\" as alias)",
            app
        ))
        .stdin(Stdio::null())
        .output()
        .context("Failed to execute osascript")?;

    let output =
        String::from_utf8(output.stdout).context("Failed to parse osascript output as UTF-8")?;

    Ok(output.trim().to_string())
}
