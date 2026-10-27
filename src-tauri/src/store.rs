// Steam store and Web API lookups: descriptions and screenshots, plus owned games when a key is set.
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

fn client() -> &'static reqwest::Client {
    static C: OnceLock<reqwest::Client> = OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Lobby/1.0 (Windows)")
            .build()
            .expect("http client")
    })
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub appid: u32,
    pub description: Option<String>,
    pub screenshots: Vec<String>,
    pub store_url: String,
    pub is_free: bool,
    pub in_store: bool,
}

static CACHE: Mutex<Option<HashMap<u32, Details>>> = Mutex::new(None);

/// Store blurbs and screenshots for up to ~50 apps per request.
pub async fn details(ids: &[u32]) -> Result<Vec<Details>, String> {
    let missing: Vec<u32> = {
        let cache = CACHE.lock().unwrap();
        ids.iter().copied().filter(|id| !cache.as_ref().is_some_and(|c| c.contains_key(id))).collect()
    };
    for chunk in missing.chunks(50) {
        let input = json!({
            "ids": chunk.iter().map(|id| json!({ "appid": id })).collect::<Vec<_>>(),
            "context": { "language": "english", "country_code": "US" },
            "data_request": { "include_basic_info": true, "include_screenshots": true }
        });
        let url = format!(
            "https://api.steampowered.com/IStoreBrowseService/GetItems/v1?input_json={}",
            urlencoding::encode(&input.to_string())
        );
        let v: Value = client().get(url).send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;
        let mut cache = CACHE.lock().unwrap();
        let cache = cache.get_or_insert_with(HashMap::new);
        for &id in chunk {
            cache.insert(id, Details { appid: id, store_url: format!("https://store.steampowered.com/app/{id}/"), ..Default::default() });
        }
        for item in v["response"]["store_items"].as_array().into_iter().flatten() {
            let Some(id) = item["appid"].as_u64().map(|i| i as u32) else { continue };
            let shots = item["screenshots"]["all_ages_screenshots"].as_array().into_iter().flatten();
            cache.insert(
                id,
                Details {
                    appid: id,
                    description: item["basic_info"]["short_description"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                    screenshots: shots
                        .filter_map(|s| s["filename"].as_str())
                        .take(8)
                        .map(|f| format!("https://shared.akamai.steamstatic.com/store_item_assets/{f}"))
                        .collect(),
                    store_url: format!("https://store.steampowered.com/app/{id}/"),
                    is_free: item["is_free"].as_bool().unwrap_or(false),
                    in_store: item["success"].as_u64() == Some(1) && item["visible"].as_bool().unwrap_or(false),
                },
            );
        }
    }
    let cache = CACHE.lock().unwrap();
    Ok(ids.iter().filter_map(|id| cache.as_ref().and_then(|c| c.get(id)).cloned()).collect())
}

/// Every game an account owns, including ones never played on this PC. Needs a Steam Web API key.
pub async fn owned_games(key: &str, steam_id: &str) -> Result<Vec<Value>, String> {
    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=1&include_played_free_games=1&skip_unvetted_apps=0",
        urlencoding::encode(key.trim()),
        urlencoding::encode(steam_id)
    );
    let res = client().get(url).send().await.map_err(|e| e.to_string())?;
    match res.status().as_u16() {
        401 | 403 => return Err("Steam rejected the Web API key.".into()),
        s if s >= 400 => return Err(format!("Steam Web API returned {s}.")),
        _ => {}
    }
    let v: Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(v["response"]["games"].as_array().cloned().unwrap_or_default())
}
