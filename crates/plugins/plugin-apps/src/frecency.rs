use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    count: u32,
    last_used: u64,
}

pub struct FrecencyStore {
    path: PathBuf,
    data: Mutex<HashMap<String, Entry>>,
}

impl FrecencyStore {
    pub fn new(path: PathBuf) -> Arc<Self> {
        let data = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Arc::new(Self {
            path,
            data: Mutex::new(data),
        })
    }

    #[cfg(test)]
    pub fn new_for_test() -> Arc<Self> {
        Arc::new(Self {
            path: PathBuf::from("/dev/null"),
            data: Mutex::new(HashMap::new()),
        })
    }

    pub fn load() -> Arc<Self> {
        let Some(data_home) = xdg::BaseDirectories::new().get_data_home() else {
            tracing::warn!("XDG_DATA_HOME unavailable; frecency disabled (in-memory only)");
            return Arc::new(Self {
                path: PathBuf::from("/dev/null"),
                data: Mutex::new(HashMap::new()),
            });
        };
        let path = data_home.join("k-launcher").join("frecency.json");
        Self::new(path)
    }

    pub fn record(&self, id: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let json = {
            let mut data = self.data.lock().unwrap();
            let entry = data.entry(id.to_string()).or_insert(Entry {
                count: 0,
                last_used: 0,
            });
            entry.count += 1;
            entry.last_used = now;
            serde_json::to_string(&*data).ok()
        }; // lock released here
        if let Some(json) = json {
            if let Some(parent) = self.path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&self.path, json);
        }
    }

    pub fn frecency_score(&self, id: &str) -> u32 {
        let data = self.data.lock().unwrap();
        let Some(entry) = data.get(id) else { return 0 };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let age_secs = now.saturating_sub(entry.last_used);
        entry.count * decay_factor(age_secs)
    }

    pub fn top_ids(&self, n: usize) -> Vec<String> {
        let data = self.data.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut scored: Vec<(String, u32)> = data
            .iter()
            .map(|(id, entry)| {
                let age_secs = now.saturating_sub(entry.last_used);
                (id.clone(), entry.count * decay_factor(age_secs))
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().take(n).map(|(id, _)| id).collect()
    }
}

fn decay_factor(age_secs: u64) -> u32 {
    if age_secs < 3600 {
        4
    } else if age_secs < 86400 {
        2
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> Arc<FrecencyStore> {
        Arc::new(FrecencyStore {
            path: PathBuf::from("/dev/null"),
            data: Mutex::new(HashMap::new()),
        })
    }

    #[test]
    fn record_increments_count() {
        let store = make_store();
        store.record("app-firefox");
        store.record("app-firefox");
        let data = store.data.lock().unwrap();
        assert_eq!(data["app-firefox"].count, 2);
    }

    #[test]
    fn record_updates_last_used() {
        let store = make_store();
        store.record("app-firefox");
        let data = store.data.lock().unwrap();
        assert!(data["app-firefox"].last_used > 0);
    }

    #[test]
    fn top_ids_returns_sorted_order() {
        let store = make_store();
        store.record("app-firefox");
        store.record("app-code");
        store.record("app-code");
        store.record("app-code");
        let top = store.top_ids(2);
        assert_eq!(top[0], "app-code");
        assert_eq!(top[1], "app-firefox");
    }
}
