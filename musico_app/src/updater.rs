//! Auto-update: checks GitHub Releases API for new versions,
//! downloads the binary, and replaces the running executable.

const REPO_OWNER: &str = "ARYAN007H";
const REPO_NAME: &str = "Musico";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Checks GitHub for a newer release.
/// Returns `Ok(Some((version, asset_url)))` if an update is available,
/// `Ok(None)` if already on latest, or `Err` on network/parse failure.
pub async fn check_for_update() -> Result<Option<(String, String)>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let client = reqwest::Client::builder()
        .user_agent(format!("Musico/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err("No releases found on GitHub".into());
    }

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let tag = body["tag_name"]
        .as_str()
        .ok_or("Missing tag_name in release")?
        .trim_start_matches('v')
        .to_string();

    // Compare versions
    if !is_newer(&tag, CURRENT_VERSION) {
        return Ok(None);
    }

    // Find the Linux x86_64 asset
    let assets = body["assets"]
        .as_array()
        .ok_or("Missing assets in release")?;

    let asset_url = assets
        .iter()
        .find(|a| {
            let name = a["name"].as_str().unwrap_or("");
            name.contains("linux") && (name.contains("x86_64") || name.contains("amd64"))
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .ok_or("No Linux x86_64 binary found in release assets")?
        .to_string();

    Ok(Some((tag, asset_url)))
}

/// Downloads the update binary and replaces the current executable.
/// User data (in ~/.config/musico/ and ~/.local/share/musico/) is untouched.
pub async fn download_and_install(url: String) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent(format!("Musico/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Download returned {}", resp.status()));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {e}"))?;

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Cannot find current executable: {e}"))?;

    let parent = current_exe
        .parent()
        .ok_or("Cannot determine executable directory")?;

    let tmp_path = parent.join(".musico_update_tmp");
    let bak_path = parent.join(".musico_update_bak");

    // Write new binary to temp file
    std::fs::write(&tmp_path, &bytes)
        .map_err(|e| format!("Failed to write update: {e}"))?;

    // Make executable (chmod +x)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&tmp_path, perms)
            .map_err(|e| format!("Failed to set permissions: {e}"))?;
    }

    // Atomic swap: current → backup, new → current
    if bak_path.exists() {
        let _ = std::fs::remove_file(&bak_path);
    }

    std::fs::rename(&current_exe, &bak_path)
        .map_err(|e| format!("Failed to backup current binary: {e}"))?;

    std::fs::rename(&tmp_path, &current_exe)
        .map_err(|e| {
            // Rollback: restore backup
            let _ = std::fs::rename(&bak_path, &current_exe);
            format!("Failed to install update: {e}")
        })?;

    // Clean up backup
    let _ = std::fs::remove_file(&bak_path);

    Ok(())
}

/// Simple semver comparison: returns true if `remote` is newer than `current`.
fn is_newer(remote: &str, current: &str) -> bool {
    let parse = |s: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = s.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(remote) > parse(current)
}
