use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::frecency::FrecencyStore;
use crate::types::{AppName, DesktopEntrySource};

pub struct CachedEntry {
    pub(crate) id: String,
    pub(crate) name: AppName,
    pub(crate) name_lowercase: String,
    pub(crate) keywords_lowercase: Vec<String>,
    pub(crate) category: Option<Arc<str>>,
    pub(crate) icon: Option<Arc<str>>,
    pub(crate) exec: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CachedEntryData {
    id: String,
    name: String,
    name_lowercase: String,
    keywords_lowercase: Vec<String>,
    category: Option<String>,
    icon: Option<String>,
    exec: String,
}

pub fn cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("k-launcher/apps.bin"))
}

pub fn load_from_path(path: &Path) -> Option<HashMap<String, CachedEntry>> {
    let data = std::fs::read(path).ok()?;
    let (entries_data, _): (Vec<CachedEntryData>, _) =
        bincode::serde::decode_from_slice(&data, bincode::config::standard()).ok()?;
    let map = entries_data
        .into_iter()
        .map(|e| {
            let cached = CachedEntry {
                id: e.id.clone(),
                name: AppName::new(e.name),
                name_lowercase: e.name_lowercase,
                keywords_lowercase: e.keywords_lowercase,
                category: e.category.map(Arc::from),
                icon: e.icon.map(Arc::from),
                exec: e.exec,
            };
            (e.id, cached)
        })
        .collect();
    Some(map)
}

pub fn save_to_path(path: &Path, entries: &HashMap<String, CachedEntry>) {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    let data: Vec<CachedEntryData> = entries
        .values()
        .map(|e| CachedEntryData {
            id: e.id.clone(),
            name: e.name.as_str().to_string(),
            name_lowercase: e.name_lowercase.clone(),
            keywords_lowercase: e.keywords_lowercase.clone(),
            category: e.category.as_deref().map(str::to_string),
            icon: e.icon.as_deref().map(str::to_string),
            exec: e.exec.clone(),
        })
        .collect();
    if let Ok(encoded) = bincode::serde::encode_to_vec(&data, bincode::config::standard()) {
        std::fs::write(path, encoded).ok();
    }
}

pub fn build_entries(
    source: &impl DesktopEntrySource,
    _frecency: &Arc<FrecencyStore>,
) -> HashMap<String, CachedEntry> {
    source
        .entries()
        .into_iter()
        .map(|e| {
            let id = format!("app-{}:{}", e.name.as_str(), e.exec.as_str());
            let name_lowercase = e.name.as_str().to_lowercase();
            let keywords_lowercase = e.keywords.iter().map(|k| k.to_lowercase()).collect();
            #[cfg(target_os = "linux")]
            let icon: Option<Arc<str>> = e
                .icon
                .as_ref()
                .and_then(|p| crate::linux::resolve_icon_path(p.as_str()))
                .map(Arc::from);
            #[cfg(not(target_os = "linux"))]
            let icon: Option<Arc<str>> = None;
            let exec = e.exec.as_str().to_string();
            let cached = CachedEntry {
                id: id.clone(),
                name_lowercase,
                keywords_lowercase,
                category: e.category.map(Arc::from),
                icon,
                exec,
                name: e.name,
            };
            (id, cached)
        })
        .collect()
}
