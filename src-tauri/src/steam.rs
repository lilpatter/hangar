// Reads the local Steam install (accounts, library, artwork) and switches the signed-in account.
use crate::vdf::{self, Node};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const STEAM_ID_BASE: u64 = 76561197960265728;
/// Custom protocol registered in main.rs; WebView2 serves it as http://art.localhost/.
pub const ART_BASE: &str = "http://art.localhost";
const CDN: &str = "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps";

pub fn hidden(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut c = Command::new(program);
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

fn steam_key(write: bool) -> Option<RegKey> {
    let flags = if write { KEY_READ | KEY_WRITE } else { KEY_READ };
    RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(r"Software\Valve\Steam", flags).ok()
}

pub fn root() -> Result<PathBuf, String> {
    let from_reg = steam_key(false)
        .and_then(|k| k.get_value::<String, _>("SteamPath").ok())
        .map(|p| PathBuf::from(p.replace('/', "\\")));
    let path = from_reg.unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)\Steam"));
    if path.join("steam.exe").exists() {
        Ok(path)
    } else {
        Err("Steam isn’t installed for this Windows user.".into())
    }
}

fn read_vdf(path: &Path) -> Result<Node, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    vdf::parse(&text)
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Persona names can carry invisible tag characters and direction overrides; drop them for display.
fn clean_name(s: &str) -> String {
    let out: String = s
        .chars()
        .filter(|&c| !matches!(c as u32, 0xE0000..=0xE007F | 0x202A..=0x202E | 0x2066..=0x2069 | 0x200B..=0x200F | 0xFEFF))
        .collect();
    out.trim().to_string()
}

// ---------- Process state ----------

mod win {
    use windows_sys::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    /// True when `pid` is a live process whose executable is steam.exe.
    pub fn is_steam(pid: u32) -> bool {
        if pid == 0 {
            return false;
        }
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() {
                return false;
            }
            let mut code = 0u32;
            let alive = GetExitCodeProcess(h, &mut code) != 0 && code == STILL_ACTIVE as u32;
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let named = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len) != 0;
            CloseHandle(h);
            alive && named && String::from_utf16_lossy(&buf[..len as usize]).to_ascii_lowercase().ends_with("\\steam.exe")
        }
    }
}

fn steam_running_anywhere() -> bool {
    hidden("tasklist")
        .args(["/FI", "IMAGENAME eq steam.exe", "/NH", "/FO", "CSV"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_ascii_lowercase().contains("\"steam.exe\""))
        .unwrap_or(false)
}

#[derive(Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub running: bool,
    /// Account ID (the low 32 bits of the SteamID) currently signed in, or 0.
    pub active_account_id: u32,
    pub auto_login_user: String,
}

pub fn status() -> Status {
    let active = steam_key(false).and_then(|k| k.open_subkey("ActiveProcess").ok());
    let pid: u32 = active.as_ref().and_then(|k| k.get_value("pid").ok()).unwrap_or(0);
    let user: u32 = active.as_ref().and_then(|k| k.get_value("ActiveUser").ok()).unwrap_or(0);
    let running = win::is_steam(pid);
    Status {
        running,
        active_account_id: if running { user } else { 0 },
        auto_login_user: steam_key(false).and_then(|k| k.get_value("AutoLoginUser").ok()).unwrap_or_default(),
    }
}

// ---------- Accounts ----------

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub steam_id: String,
    pub account_id: u32,
    pub account_name: String,
    pub persona_name: String,
    pub remember_password: bool,
    pub last_login: u64,
    pub avatar: Option<String>,
}

pub fn accounts(root: &Path) -> Vec<Account> {
    let Ok(users) = read_vdf(&root.join(r"config\loginusers.vdf")) else { return Vec::new() };
    let mut list: Vec<Account> = users
        .get("users")
        .map(|u| u.entries())
        .unwrap_or(&[])
        .iter()
        .filter_map(|(id, u)| {
            let sid: u64 = id.parse().ok()?;
            let account_name = u.str("AccountName")?.to_string();
            let avatar = root.join(format!(r"config\avatarcache\{id}.png")).exists().then(|| format!("{ART_BASE}/avatar/{id}"));
            Some(Account {
                steam_id: id.clone(),
                account_id: (sid.wrapping_sub(STEAM_ID_BASE)) as u32,
                persona_name: u.str("PersonaName").map(clean_name).filter(|s| !s.is_empty()).unwrap_or_else(|| account_name.clone()),
                account_name,
                remember_password: u.str("RememberPassword") == Some("1"),
                last_login: u.num("Timestamp").unwrap_or(0),
                avatar,
            })
        })
        .collect();
    list.sort_by(|a, b| b.last_login.cmp(&a.last_login));
    list
}

// ---------- Library ----------

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Ownership {
    pub account_id: u32,
    pub playtime: u64,
    pub last_played: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogoPosition {
    pub pinned: String,
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub appid: u32,
    pub name: String,
    /// "game", "software" or "demo".
    pub kind: String,
    /// "installed", "update", "downloading" or "none".
    pub state: String,
    pub size_on_disk: u64,
    pub install_path: Option<String>,
    pub owners: Vec<Ownership>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release: Option<u64>,
    pub genres: Vec<String>,
    pub review_percent: Option<u64>,
    pub metacritic: Option<u64>,
    pub controller: Option<String>,
    pub cover: Option<String>,
    pub hero: Option<String>,
    pub logo: Option<String>,
    pub header: Option<String>,
    pub icon: Option<String>,
    pub logo_position: Option<LogoPosition>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub steam_path: String,
    pub accounts: Vec<Account>,
    pub games: Vec<Game>,
    pub folders: Vec<String>,
    pub status: Status,
}

struct Manifest {
    name: String,
    flags: u64,
    size: u64,
    path: PathBuf,
    last_owner: Option<u64>,
}

pub fn library_folders(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    if let Ok(v) = read_vdf(&root.join(r"steamapps\libraryfolders.vdf")) {
        let top = v.get("libraryfolders").or(v.get("LibraryFolders"));
        for (_, f) in top.map(|t| t.entries()).unwrap_or(&[]) {
            let p = match f {
                Node::Obj(_) => f.str("path").map(PathBuf::from),
                Node::Str(s) => Some(PathBuf::from(s)),
            };
            if let Some(p) = p.filter(|p| p.join("steamapps").is_dir()) {
                if !out.iter().any(|o| o.as_os_str().eq_ignore_ascii_case(p.as_os_str())) {
                    out.push(p);
                }
            }
        }
    }
    out
}

fn manifests(folders: &[PathBuf]) -> HashMap<u32, Manifest> {
    let mut out = HashMap::new();
    for lib in folders {
        let apps = lib.join("steamapps");
        let Ok(dir) = std::fs::read_dir(&apps) else { continue };
        for e in dir.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !(name.starts_with("appmanifest_") && name.ends_with(".acf")) {
                continue;
            }
            let Ok(v) = read_vdf(&e.path()) else { continue };
            let Some(state) = v.get("AppState") else { continue };
            let Some(appid) = state.num("appid") else { continue };
            out.insert(
                appid as u32,
                Manifest {
                    name: state.str("name").unwrap_or_default().to_string(),
                    flags: state.num("StateFlags").unwrap_or(0),
                    size: state.num("SizeOnDisk").unwrap_or(0),
                    path: apps.join("common").join(state.str("installdir").unwrap_or_default()),
                    last_owner: state.num("LastOwner").filter(|&o| o > STEAM_ID_BASE),
                },
            );
        }
    }
    out
}

/// Apps an account has used on this PC, with playtime (minutes) and last played time.
fn account_apps(root: &Path, account_id: u32) -> HashMap<u32, Ownership> {
    let base = root.join(format!(r"userdata\{account_id}"));
    let mut out: HashMap<u32, Ownership> = HashMap::new();
    let mut add = |id: u32, playtime: u64, last: u64| {
        let o = out.entry(id).or_insert_with(|| Ownership { account_id, ..Default::default() });
        o.playtime = o.playtime.max(playtime);
        o.last_played = o.last_played.max(last);
    };
    if let Ok(v) = read_vdf(&base.join(r"config\localconfig.vdf")) {
        if let Some(apps) = v.path(&["UserLocalConfigStore", "Software", "Valve", "Steam", "apps"]) {
            for (id, a) in apps.entries() {
                if let Ok(id) = id.parse() {
                    add(id, a.num("Playtime").unwrap_or(0), a.num("LastPlayed").unwrap_or(0));
                }
            }
        }
    }
    if let Ok(v) = read_vdf(&base.join(r"7\remote\sharedconfig.vdf")) {
        if let Some(apps) = v.path(&["UserRoamingConfigStore", "Software", "Valve", "Steam", "apps"]) {
            for (id, _) in apps.entries() {
                if let Ok(id) = id.parse() {
                    add(id, 0, 0);
                }
            }
        }
    }
    // The client keeps a small per-account cache for every app opened in its library.
    if let Ok(dir) = std::fs::read_dir(base.join(r"config\librarycache")) {
        for e in dir.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(id) = name.strip_suffix(".json").and_then(|s| s.parse().ok()) {
                add(id, 0, 0);
            }
        }
    }
    out
}

const GENRES: &[(&str, &str)] = &[
    ("1", "Action"), ("2", "Strategy"), ("3", "RPG"), ("4", "Casual"), ("9", "Racing"), ("18", "Sports"),
    ("23", "Indie"), ("25", "Adventure"), ("28", "Simulation"), ("29", "Massively Multiplayer"), ("37", "Free to Play"),
    ("51", "Animation & Modeling"), ("52", "Audio Production"), ("53", "Design & Illustration"), ("54", "Education"),
    ("55", "Photo Editing"), ("56", "Software Training"), ("57", "Utilities"), ("58", "Video Production"),
    ("59", "Web Publishing"), ("60", "Game Development"), ("70", "Early Access"),
];

fn kind_of(t: &str) -> Option<&'static str> {
    match t.to_ascii_lowercase().as_str() {
        "game" => Some("game"),
        "application" => Some("software"),
        "demo" => Some("demo"),
        _ => None,
    }
}

/// Files cached by the Steam client for one app, relative to appcache\librarycache\<appid>.
fn cached_files(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return out };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if e.path().is_dir() {
            if let Ok(sub) = std::fs::read_dir(e.path()) {
                out.extend(sub.flatten().map(|s| format!("{name}/{}", s.file_name().to_string_lossy())));
            }
        } else {
            out.push(name);
        }
    }
    out
}

fn english(n: Option<&Node>) -> Option<String> {
    let n = n?;
    match n {
        Node::Str(s) => Some(s.clone()),
        Node::Obj(_) => n.str("english").or_else(|| n.entries().first().and_then(|(_, v)| if let Node::Str(s) = v { Some(s.as_str()) } else { None })).map(str::to_string),
    }
}

/// Local copy when the client has it cached, otherwise Steam's CDN.
fn artwork(appid: u32, files: &[String], common: Option<&Node>, game: &mut Game) {
    let full = common.and_then(|c| c.get("library_assets_full"));
    let pick = |key: &str, names: &[&str]| -> Option<String> {
        let rel = english(full.and_then(|f| f.path(&[key, "image"])));
        if let Some(rel) = &rel {
            if files.iter().any(|f| f == rel) {
                return Some(format!("{ART_BASE}/{appid}/{rel}"));
            }
        }
        if let Some(f) = files.iter().find(|f| names.iter().any(|n| f.rsplit('/').next() == Some(n))) {
            return Some(format!("{ART_BASE}/{appid}/{f}"));
        }
        // Apps without the newer asset list still use the classic file names on the CDN.
        let rel = rel.or_else(|| (full.is_none()).then(|| names[0].to_string()))?;
        Some(format!("{CDN}/{appid}/{rel}"))
    };
    game.cover = pick("library_capsule", &["library_600x900.jpg", "library_capsule.jpg"]);
    game.hero = pick("library_hero", &["library_hero.jpg"]);
    game.logo = pick("library_logo", &["logo.png"]);
    game.header = pick("library_header", &["header.jpg", "library_header.jpg"]).or_else(|| {
        english(common.and_then(|c| c.get("header_image"))).map(|rel| format!("{CDN}/{appid}/{rel}"))
    });
    if let Some(hash) = common.and_then(|c| c.str("icon")) {
        let local = format!("{hash}.jpg");
        game.icon = Some(if files.contains(&local) {
            format!("{ART_BASE}/{appid}/{local}")
        } else {
            format!("https://cdn.fastly.steamstatic.com/steamcommunity/public/images/apps/{appid}/{hash}.jpg")
        });
    }
    let pos = full.and_then(|f| f.path(&["library_logo", "logo_position"])).or_else(|| common.and_then(|c| c.path(&["library_assets", "logo_position"])));
    if let Some(p) = pos {
        game.logo_position = Some(LogoPosition {
            pinned: p.str("pinned_position").unwrap_or("BottomLeft").to_string(),
            width: p.str("width_pct").and_then(|s| s.parse().ok()).unwrap_or(40.0),
            height: p.str("height_pct").and_then(|s| s.parse().ok()).unwrap_or(40.0),
        });
    }
}

fn describe(common: &Node, game: &mut Game) {
    for (_, a) in common.get("associations").map(|a| a.entries()).unwrap_or(&[]) {
        let name = a.str("name").map(str::to_string);
        match a.str("type") {
            Some("developer") if game.developer.is_none() => game.developer = name,
            Some("publisher") if game.publisher.is_none() => game.publisher = name,
            _ => {}
        }
    }
    game.release = common.num("steam_release_date").or_else(|| common.num("original_release_date")).filter(|&t| t > 0);
    game.genres = common
        .get("genres")
        .map(|g| g.entries())
        .unwrap_or(&[])
        .iter()
        .filter_map(|(_, v)| if let Node::Str(id) = v { GENRES.iter().find(|(g, _)| g == id).map(|(_, n)| n.to_string()) } else { None })
        .collect();
    game.review_percent = common.num("review_percentage").filter(|&p| p > 0);
    game.metacritic = common.num("metacritic_score").filter(|&p| p > 0);
    game.controller = common.str("controller_support").map(str::to_string);
}

pub fn library() -> Result<Library, String> {
    let root = root()?;
    let accounts = accounts(&root);
    let folders = library_folders(&root);
    let installed = manifests(&folders);
    let appinfo = std::fs::read(root.join(r"appcache\appinfo.vdf"))
        .ok()
        .and_then(|d| vdf::parse_appinfo(&d).ok())
        .unwrap_or_default();

    let mut owners: HashMap<u32, Vec<Ownership>> = HashMap::new();
    for a in &accounts {
        for (id, o) in account_apps(&root, a.account_id) {
            owners.entry(id).or_default().push(o);
        }
    }
    for (id, m) in &installed {
        if let Some(sid) = m.last_owner {
            let account_id = (sid - STEAM_ID_BASE) as u32;
            let list = owners.entry(*id).or_default();
            if !list.iter().any(|o| o.account_id == account_id) {
                list.push(Ownership { account_id, ..Default::default() });
            }
        }
    }

    // appinfo.vdf holds every app the client has licence info for on this PC; manifests cover the rest.
    let ids: HashSet<u32> = appinfo.keys().chain(installed.keys()).copied().collect();
    let cache = root.join(r"appcache\librarycache");
    let mut games: BTreeMap<String, Game> = BTreeMap::new();
    for id in ids {
        let common = appinfo.get(&id);
        let kind = match common {
            Some(c) => match c.str("type").and_then(kind_of) {
                Some(k) => k,
                None => continue,
            },
            None if installed.contains_key(&id) && id != 228980 => "game",
            None => continue,
        };
        let manifest = installed.get(&id);
        let name = common.and_then(|c| c.str("name")).map(str::to_string).or_else(|| manifest.map(|m| m.name.clone())).unwrap_or_default();
        if name.trim().is_empty() {
            continue;
        }
        let mut g = Game { appid: id, name: name.trim().to_string(), kind: kind.into(), state: "none".into(), ..Default::default() };
        if let Some(m) = manifest {
            g.state = if m.flags & 0x0010_0000 != 0 || m.flags & 0x0000_0500 != 0 {
                "downloading"
            } else if m.flags & 4 == 0 {
                "downloading"
            } else if m.flags & 2 != 0 {
                "update"
            } else {
                "installed"
            }
            .into();
            g.size_on_disk = m.size;
            g.install_path = Some(m.path.display().to_string());
        }
        g.owners = owners.remove(&id).unwrap_or_default();
        if let Some(c) = common {
            describe(c, &mut g);
        }
        artwork(id, &cached_files(&cache.join(id.to_string())), common, &mut g);
        games.insert(format!("{}\u{0}{id}", g.name.to_lowercase()), g);
    }

    Ok(Library {
        steam_path: root.display().to_string(),
        accounts,
        games: games.into_values().collect(),
        folders: folders.iter().map(|f| f.display().to_string()).collect(),
        status: status(),
    })
}

/// Adds games from the Steam Web API (needs a key) so never-played games show up too.
pub fn merge_owned(lib: &mut Library, account_id: u32, owned: &[serde_json::Value]) {
    for o in owned {
        let Some(id) = o["appid"].as_u64().map(|i| i as u32) else { continue };
        let own = Ownership {
            account_id,
            playtime: o["playtime_forever"].as_u64().unwrap_or(0),
            last_played: o["rtime_last_played"].as_u64().unwrap_or(0),
        };
        match lib.games.iter_mut().find(|g| g.appid == id) {
            Some(g) => match g.owners.iter_mut().find(|x| x.account_id == account_id) {
                Some(x) => {
                    x.playtime = x.playtime.max(own.playtime);
                    x.last_played = x.last_played.max(own.last_played);
                }
                None => g.owners.push(own),
            },
            None => {
                let Some(name) = o["name"].as_str().filter(|n| !n.trim().is_empty()) else { continue };
                let mut g = Game { appid: id, name: name.trim().into(), kind: "game".into(), state: "none".into(), owners: vec![own], ..Default::default() };
                artwork(id, &[], None, &mut g);
                if let Some(hash) = o["img_icon_url"].as_str().filter(|h| !h.is_empty()) {
                    g.icon = Some(format!("https://cdn.fastly.steamstatic.com/steamcommunity/public/images/apps/{id}/{hash}.jpg"));
                }
                lib.games.push(g);
            }
        }
    }
    lib.games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
}

// ---------- Local artwork protocol ----------

/// Resolves /<appid>/<file> and /avatar/<steamid> to files in Steam's own caches.
pub fn art_file(path: &str) -> Option<PathBuf> {
    let root = root().ok()?;
    let path = urlencoding::decode(path.trim_start_matches('/')).ok()?.into_owned();
    if path.split('/').any(|p| p.is_empty() || p == "." || p == ".." || p.contains(['\\', ':'])) {
        return None;
    }
    let file = match path.split_once('/') {
        Some(("avatar", id)) if id.chars().all(|c| c.is_ascii_digit()) => root.join(format!(r"config\avatarcache\{id}.png")),
        Some((app, rest)) if app.chars().all(|c| c.is_ascii_digit()) => root.join(r"appcache\librarycache").join(app).join(rest.replace('/', "\\")),
        _ => return None,
    };
    file.is_file().then_some(file)
}

// ---------- Switching ----------

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwitchResult {
    pub signed_in: bool,
    pub needs_password: bool,
}

pub struct SwitchOptions {
    pub show_window: bool,
    pub offline: bool,
    pub launch: Option<u32>,
}

fn shutdown(root: &Path) -> Result<(), String> {
    let s = status();
    if !s.running && !steam_running_anywhere() {
        return Ok(());
    }
    let _ = hidden(root.join("steam.exe")).arg("-shutdown").spawn();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        std::thread::sleep(Duration::from_millis(400));
        if !steam_running_anywhere() {
            // Give the client a moment to finish writing its config files.
            std::thread::sleep(Duration::from_millis(600));
            return Ok(());
        }
    }
    let _ = hidden("taskkill").args(["/F", "/T", "/IM", "steam.exe"]).output();
    std::thread::sleep(Duration::from_millis(1200));
    if steam_running_anywhere() {
        return Err("Steam didn’t close. Quit Steam from its tray icon and try again.".into());
    }
    Ok(())
}

fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    let tmp = path.with_extension("vdf.tmp");
    std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

/// Marks one remembered account as the auto-login account, the same way Steam's own account picker does.
fn prepare_login(root: &Path, account_name: Option<&str>, offline: bool) -> Result<(), String> {
    let file = root.join(r"config\loginusers.vdf");
    let mut doc = read_vdf(&file)?;
    select_login_user(&mut doc, account_name, offline)?;
    write_atomic(&file, &vdf::write(&doc))?;

    let key = steam_key(true).ok_or("Couldn’t open Steam’s registry settings.")?;
    key.set_value("AutoLoginUser", &account_name.unwrap_or("").to_string()).map_err(|e| e.to_string())?;
    key.set_value("RememberPassword", &1u32).map_err(|e| e.to_string())?;
    Ok(())
}

/// Newer clients flag the auto-login account with "AutoLogin", older ones with "MostRecent"/"AllowAutoLogin".
fn select_login_user(doc: &mut Node, account_name: Option<&str>, offline: bool) -> Result<(), String> {
    let users = doc.get_mut("users").ok_or("loginusers.vdf has no users")?;
    let entries = users.entries_mut().ok_or("loginusers.vdf has no users")?;
    let new_format = entries.iter().any(|(_, u)| u.has("AutoLogin"));
    let mut found = false;
    for (_, u) in entries.iter_mut() {
        let me = account_name.is_some_and(|n| u.str("AccountName").is_some_and(|a| a.eq_ignore_ascii_case(n)));
        found |= me;
        let flag = if me { "1" } else { "0" };
        if new_format || u.has("AutoLogin") {
            u.set("AutoLogin", flag);
        }
        if !new_format || u.has("MostRecent") {
            u.set("MostRecent", flag);
        }
        if me {
            u.set("RememberPassword", "1");
            if u.has("AllowAutoLogin") || !new_format {
                u.set("AllowAutoLogin", "1");
            }
            u.set("WantsOfflineMode", if offline { "1" } else { "0" });
            u.set("SkipOfflineModeWarning", "1");
            u.set("Timestamp", &now().to_string());
        }
    }
    if account_name.is_some() && !found {
        return Err("That account isn’t remembered by Steam on this PC.".into());
    }
    Ok(())
}

fn launch_steam(root: &Path, show_window: bool, url: Option<String>) -> Result<(), String> {
    let mut cmd = Command::new(root.join("steam.exe"));
    cmd.current_dir(root);
    if !show_window {
        cmd.arg("-silent");
    }
    if let Some(u) = url {
        cmd.arg(u);
    }
    cmd.spawn().map(|_| ()).map_err(|e| format!("Couldn’t start Steam: {e}"))
}

pub fn switch(account_name: &str, opts: SwitchOptions, progress: impl Fn(&str)) -> Result<SwitchResult, String> {
    let root = root()?;
    let account = accounts(&root).into_iter().find(|a| a.account_name.eq_ignore_ascii_case(account_name)).ok_or("That account isn’t remembered by Steam on this PC.")?;
    let s = status();
    if s.running && s.active_account_id == account.account_id {
        if let Some(id) = opts.launch {
            open_url(&format!("steam://rungameid/{id}"))?;
        }
        return Ok(SwitchResult { signed_in: true, needs_password: false });
    }
    progress("closing");
    shutdown(&root)?;
    progress("preparing");
    prepare_login(&root, Some(&account.account_name), opts.offline)?;
    progress("starting");
    launch_steam(&root, opts.show_window, opts.launch.map(|id| format!("steam://rungameid/{id}")))?;
    progress("signing-in");
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(60) {
        std::thread::sleep(Duration::from_millis(700));
        let s = status();
        if s.running && s.active_account_id == account.account_id {
            return Ok(SwitchResult { signed_in: true, needs_password: false });
        }
    }
    Ok(SwitchResult { signed_in: false, needs_password: !account.remember_password })
}

/// Opens Steam's sign-in window for an account that isn't remembered yet.
pub fn add_account(progress: impl Fn(&str)) -> Result<(), String> {
    let root = root()?;
    progress("closing");
    shutdown(&root)?;
    progress("preparing");
    prepare_login(&root, None, false)?;
    progress("starting");
    launch_steam(&root, true, None)
}

pub fn open_url(url: &str) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("steam://")) {
        return Err("Only https and steam links can be opened.".into());
    }
    hidden("rundll32.exe").args(["url.dll,FileProtocolHandler", url]).spawn().map(|_| ()).map_err(|e| e.to_string())
}

pub fn reveal(path: &str) -> Result<(), String> {
    let root = root()?;
    let p = PathBuf::from(path);
    let inside = library_folders(&root).iter().any(|f| p.starts_with(f.join("steamapps")));
    if !inside || !p.is_dir() {
        return Err("That folder isn’t in a Steam library.".into());
    }
    hidden("explorer.exe").arg(p).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEW: &str = "\"users\"\n{\n\t\"76561197960265729\"\n\t{\n\t\t\"AccountName\"\t\t\"first\"\n\t\t\"PersonaName\"\t\t\"One\"\n\t\t\"RememberPassword\"\t\t\"1\"\n\t\t\"WantsOfflineMode\"\t\t\"0\"\n\t\t\"SkipOfflineModeWarning\"\t\t\"0\"\n\t\t\"AutoLogin\"\t\t\"1\"\n\t\t\"timestamp\"\t\t\"100\"\n\t}\n\t\"76561197960265730\"\n\t{\n\t\t\"AccountName\"\t\t\"second\"\n\t\t\"PersonaName\"\t\t\"Two \\\"quoted\\\"\"\n\t\t\"RememberPassword\"\t\t\"1\"\n\t\t\"AutoLogin\"\t\t\"0\"\n\t\t\"timestamp\"\t\t\"90\"\n\t}\n}\n";

    fn user<'a>(doc: &'a Node, id: &str) -> &'a Node {
        doc.path(&["users", id]).unwrap()
    }

    #[test]
    fn round_trips_loginusers() {
        let doc = vdf::parse(NEW).unwrap();
        assert_eq!(vdf::write(&doc), NEW);
        assert_eq!(user(&doc, "76561197960265730").str("PersonaName"), Some("Two \"quoted\""));
    }

    #[test]
    fn selects_account_in_new_format() {
        let mut doc = vdf::parse(NEW).unwrap();
        select_login_user(&mut doc, Some("SECOND"), false).unwrap();
        let (a, b) = (user(&doc, "76561197960265729"), user(&doc, "76561197960265730"));
        assert_eq!(a.str("AutoLogin"), Some("0"));
        assert_eq!(b.str("AutoLogin"), Some("1"));
        assert_eq!(b.str("SkipOfflineModeWarning"), Some("1"));
        assert!(!b.has("MostRecent") && !b.has("AllowAutoLogin"));
        // The existing lowercase key is updated in place, not duplicated.
        assert!(b.str("timestamp").unwrap().parse::<u64>().unwrap() > 90);
        assert_eq!(b.entries().iter().filter(|(k, _)| k.eq_ignore_ascii_case("timestamp")).count(), 1);
    }

    #[test]
    fn selects_account_in_old_format() {
        let old = NEW.replace("\"AutoLogin\"\t\t\"1\"", "\"MostRecent\"\t\t\"1\"").replace("\"AutoLogin\"\t\t\"0\"", "\"MostRecent\"\t\t\"0\"");
        let mut doc = vdf::parse(&old).unwrap();
        select_login_user(&mut doc, Some("second"), true).unwrap();
        let (a, b) = (user(&doc, "76561197960265729"), user(&doc, "76561197960265730"));
        assert_eq!(a.str("MostRecent"), Some("0"));
        assert_eq!(b.str("MostRecent"), Some("1"));
        assert_eq!(b.str("AllowAutoLogin"), Some("1"));
        assert_eq!(b.str("WantsOfflineMode"), Some("1"));
        assert!(!b.has("AutoLogin"));
    }

    #[test]
    fn rejects_unknown_account() {
        let mut doc = vdf::parse(NEW).unwrap();
        assert!(select_login_user(&mut doc, Some("nobody"), false).is_err());
    }

    #[test]
    fn reads_real_install_when_present() {
        let Ok(root) = root() else { return };
        let users = read_vdf(&root.join(r"config\loginusers.vdf")).unwrap();
        assert_eq!(vdf::parse(&vdf::write(&users)).unwrap().entries().len(), users.entries().len());
        let lib = library().unwrap();
        assert!(!lib.games.is_empty());
    }
}
