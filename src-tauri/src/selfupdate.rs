// Updates Hangar itself from the GitHub releases that CI publishes.
//
// Windows won't delete a running exe but it will rename one. So the swap is: download the new
// exe next to the current one, move the running exe aside to <name>.old, move the new one into
// its place, start it and exit. The new process deletes <name>.old once the old one has gone.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const REPO: &str = "lilpatter/hangar";
const ASSET: &str = "hangar.exe";
/// Passed to the relaunched exe so it waits for the old process before opening its window.
pub const UPDATED_ARG: &str = "--updated";
/// CI sets this to the tag the build is published under. Local and pull request builds have
/// none and never update themselves.
const TAG: Option<&str> = option_env!("HANGAR_RELEASE_TAG");

static BUSY: AtomicBool = AtomicBool::new(false);
/// Set when this process was started by an update, so the UI can say so.
static JUST_UPDATED: AtomicBool = AtomicBool::new(false);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub version: String,
    pub tag: Option<String>,
    pub just_updated: bool,
    pub latest: Option<Release>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub tag: String,
    pub name: String,
    pub url: String,
    pub size: u64,
    pub newer: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Progress {
    state: &'static str,
    received: u64,
    total: u64,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    name: Option<String>,
    html_url: String,
    assets: Vec<GhAsset>,
}

#[derive(Deserialize, Clone)]
struct GhAsset {
    name: String,
    size: u64,
    browser_download_url: String,
    /// "sha256:<hex>", filled in by GitHub for every uploaded asset.
    digest: Option<String>,
}

fn client() -> &'static reqwest::Client {
    static C: OnceLock<reqwest::Client> = OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(concat!("Hangar/", env!("CARGO_PKG_VERSION"), " (Windows)"))
            .connect_timeout(Duration::from_secs(20))
            .read_timeout(Duration::from_secs(60))
            .build()
            .expect("update client")
    })
}

fn tag() -> Option<&'static str> {
    TAG.filter(|t| !t.is_empty())
}

/// v1.2.0 → (1, 2, 0, 0); v1.2.0-build.7 → (1, 2, 0, 7).
fn parse_tag(tag: &str) -> Option<(u64, u64, u64, u64)> {
    let t = tag.trim();
    let t = t.strip_prefix('v').unwrap_or(t);
    let (version, build) = match t.split_once('-') {
        Some((v, rest)) => (v, rest.strip_prefix("build.")?.parse().ok()?),
        None => (t, 0),
    };
    let mut parts = version.split('.').map(|n| n.parse::<u64>().ok());
    let v = (parts.next()??, parts.next()??, parts.next()??, build);
    parts.next().is_none().then_some(v)
}

/// Only a strictly newer release counts, so Hangar never downgrades or loops between builds.
fn newer(tag: &str) -> bool {
    matches!((parse_tag(tag), self::tag().and_then(parse_tag)), (Some(l), Some(c)) if l > c)
}

async fn latest() -> Result<(GhRelease, GhAsset), String> {
    let res = client()
        .get(format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .header("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("Couldn't reach GitHub: {e}"))?;
    match res.status().as_u16() {
        403 | 429 => return Err("GitHub is limiting update checks right now. Hangar will try again later.".into()),
        s if s >= 400 => return Err(format!("GitHub answered with HTTP {s}.")),
        _ => {}
    }
    let rel: GhRelease = res.json().await.map_err(|e| format!("Unexpected answer from GitHub: {e}"))?;
    let asset = rel.assets.iter().find(|a| a.name.eq_ignore_ascii_case(ASSET)).cloned().ok_or("The latest release has no hangar.exe.")?;
    Ok((rel, asset))
}

pub async fn check(app: &AppHandle) -> Result<Status, String> {
    let mut status = Status {
        version: app.package_info().version.to_string(),
        tag: tag().map(String::from),
        just_updated: JUST_UPDATED.load(Ordering::Relaxed),
        latest: None,
    };
    if status.tag.is_none() {
        return Ok(status);
    }
    let (rel, asset) = latest().await?;
    status.latest = Some(Release {
        newer: newer(&rel.tag_name),
        name: rel.name.filter(|n| !n.is_empty()).unwrap_or_else(|| rel.tag_name.clone()),
        tag: rel.tag_name,
        url: rel.html_url,
        size: asset.size,
    });
    Ok(status)
}

fn sibling(exe: &Path, suffix: &str) -> PathBuf {
    let mut name = exe.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    exe.with_file_name(name)
}

/// Downloads the latest release, swaps it in and relaunches. Only returns on failure.
pub async fn install(app: &AppHandle) -> Result<(), String> {
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err("An update is already in progress.".into());
    }
    let result = install_inner(app).await;
    BUSY.store(false, Ordering::SeqCst);
    result
}

async fn install_inner(app: &AppHandle) -> Result<(), String> {
    if tag().is_none() {
        return Err("This is a local build, so it doesn't update itself.".into());
    }
    let (rel, asset) = latest().await?;
    if !newer(&rel.tag_name) {
        return Err("Hangar is already up to date.".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let staged = sibling(&exe, ".new");
    if let Err(e) = download(app, &asset, &staged).await {
        let _ = std::fs::remove_file(&staged);
        return Err(e);
    }
    let _ = app.emit("self-update", Progress { state: "restarting", received: asset.size, total: asset.size });
    swap(&exe, &staged)?;
    app.exit(0);
    Ok(())
}

async fn download(app: &AppHandle, asset: &GhAsset, to: &Path) -> Result<(), String> {
    // Only ever run a file from this repository's releases, checked against GitHub's digest.
    if !asset.browser_download_url.starts_with(&format!("https://github.com/{REPO}/releases/download/")) {
        return Err("The update isn't hosted in Hangar's GitHub releases, so it wasn't installed.".into());
    }
    let expected = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .ok_or("GitHub didn't publish a checksum for the update, so it wasn't installed.")?;

    let mut res = client().get(&asset.browser_download_url).send().await.map_err(|e| format!("Couldn't download the update: {e}"))?;
    if !res.status().is_success() {
        return Err(format!("GitHub answered with HTTP {} for the update.", res.status().as_u16()));
    }
    let mut file = std::fs::File::create(to).map_err(|e| format!("Can't write next to hangar.exe ({e})."))?;
    let mut hasher = Sha256::new();
    let mut received = 0u64;
    let mut head: Vec<u8> = Vec::with_capacity(2);
    let mut last_emit = Instant::now();
    while let Some(chunk) = res.chunk().await.map_err(|e| format!("The update download was interrupted ({e})."))? {
        if head.len() < 2 {
            head.extend(chunk.iter().take(2 - head.len()));
        }
        hasher.update(&chunk);
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        received += chunk.len() as u64;
        if last_emit.elapsed() >= Duration::from_millis(150) {
            let _ = app.emit("self-update", Progress { state: "downloading", received, total: asset.size });
            last_emit = Instant::now();
        }
    }
    file.flush().map_err(|e| e.to_string())?;
    drop(file);

    if received != asset.size {
        return Err(format!("The update download was incomplete ({received} of {} bytes).", asset.size));
    }
    if head != b"MZ" {
        return Err("The downloaded update isn't a Windows program.".into());
    }
    let hash: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if !expected.eq_ignore_ascii_case(&hash) {
        return Err("The downloaded update doesn't match GitHub's checksum. Nothing was changed.".into());
    }
    Ok(())
}

fn swap(exe: &Path, staged: &Path) -> Result<(), String> {
    let old = sibling(exe, ".old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(exe, &old).map_err(|e| format!("Couldn't move the running Hangar aside ({e})."))?;
    let restore = |err: String| {
        let _ = std::fs::remove_file(exe);
        let _ = std::fs::rename(&old, exe);
        let _ = std::fs::remove_file(staged);
        err
    };
    std::fs::rename(staged, exe).map_err(|e| restore(format!("Couldn't put the new Hangar in place ({e}).")))?;
    std::process::Command::new(exe)
        .arg(UPDATED_ARG)
        .spawn()
        .map_err(|e| restore(format!("Couldn't start the new Hangar ({e}).")))?;
    Ok(())
}

/// Removes what an earlier update left behind. After a relaunch the old process may still be
/// shutting down, and its exe can only be deleted once it has, so wait for that (briefly)
/// before the new window starts up alongside it.
pub fn cleanup() {
    let Ok(exe) = std::env::current_exe() else { return };
    let _ = std::fs::remove_file(sibling(&exe, ".new"));
    let old = sibling(&exe, ".old");
    let relaunched = std::env::args().any(|a| a == UPDATED_ARG);
    JUST_UPDATED.store(relaunched, Ordering::Relaxed);
    for _ in 0..if relaunched { 40 } else { 1 } {
        match std::fs::remove_file(&old) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => std::thread::sleep(Duration::from_millis(250)),
            _ => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_release_tags() {
        assert_eq!(parse_tag("v1.0.0-build.5"), Some((1, 0, 0, 5)));
        assert_eq!(parse_tag("v1.2.0"), Some((1, 2, 0, 0)));
        assert_eq!(parse_tag("1.2.3-beta"), None);
        assert_eq!(parse_tag("v1.2"), None);
        assert!(parse_tag("v1.0.0-build.10") > parse_tag("v1.0.0-build.9"));
        assert!(parse_tag("v1.1.0") > parse_tag("v1.0.0-build.40"));
    }
}
