use anyhow::Result;

pub fn get_app_path(app: &str) -> Result<String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "tell application \"System Events\" to POSIX path of (file of process \"{}\" as alias)",
            app
        ))
        .stdout(std::process::Stdio::piped())
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute osascript: {}", e))?;

    let output = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse osascript output: {}", e))?;

    Ok(output.trim().to_string())
}
