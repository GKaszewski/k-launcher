use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    count: u32,
    last_used: u64,
}

#[derive(Serialize, Deserialize)]
struct LogRecord {
    id: String,
    ts: u64,
}

pub struct FrecencyStore {
    snapshot_path: PathBuf,
    log_path: PathBuf,
    data: Mutex<HashMap<String, Entry>>,
    log_count: Mutex<usize>,
    compact_threshold: usize,
}

impl FrecencyStore {
    pub fn new(snapshot_path: PathBuf, compact_threshold: usize) -> Arc<Self> {
        let mut data: HashMap<String, Entry> = std::fs::read_to_string(&snapshot_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let log_path = snapshot_path.with_extension("log");
        let log_count = replay_log_into(&log_path, &mut data);

        let store = Arc::new(Self {
            snapshot_path,
            log_path,
            data: Mutex::new(data),
            log_count: Mutex::new(log_count),
            compact_threshold,
        });

        if log_count >= compact_threshold {
            store.compact();
        }

        store
    }

    pub fn new_for_test() -> Arc<Self> {
        Arc::new(Self {
            snapshot_path: PathBuf::from("/dev/null"),
            log_path: PathBuf::from("/dev/null"),
            data: Mutex::new(HashMap::new()),
            log_count: Mutex::new(0),
            compact_threshold: usize::MAX,
        })
    }

    pub fn load(compact_threshold: usize) -> Arc<Self> {
        let Some(data_home) = xdg::BaseDirectories::new().get_data_home() else {
            tracing::warn!("XDG_DATA_HOME unavailable; frecency disabled (in-memory only)");
            return Self::new_for_test();
        };
        let path = data_home
            .join(k_launcher_domain::constants::APP_NAME)
            .join(k_launcher_domain::constants::FRECENCY_SNAPSHOT_FILENAME);
        Self::new(path, compact_threshold)
    }

    pub fn record(&self, id: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        {
            let mut data = self.data.lock();
            let entry = data.entry(id.to_string()).or_insert(Entry {
                count: 0,
                last_used: 0,
            });
            entry.count += 1;
            entry.last_used = now;
        }
        self.append_log(id, now);
    }

    fn append_log(&self, id: &str, ts: u64) {
        let record = LogRecord {
            id: id.to_string(),
            ts,
        };
        if let Ok(json) = serde_json::to_string(&record) {
            if let Some(parent) = self.log_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                tracing::warn!("failed to create frecency dir: {e}");
            }
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_path)
                && let Err(e) = writeln!(file, "{json}")
            {
                tracing::warn!("failed to write frecency log: {e}");
            }
        }
        let mut count = self.log_count.lock();
        *count += 1;
        if *count >= self.compact_threshold {
            drop(count);
            self.compact();
        }
    }

    pub fn compact(&self) {
        let json = {
            let data = self.data.lock();
            serde_json::to_string(&*data).ok()
        };
        if let Some(json) = json {
            if let Some(parent) = self.snapshot_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                tracing::warn!("failed to create frecency dir: {e}");
            }
            if std::fs::write(&self.snapshot_path, json).is_ok() {
                if let Err(e) = File::create(&self.log_path) {
                    tracing::warn!("failed to truncate frecency log: {e}");
                }
                *self.log_count.lock() = 0;
            }
        }
    }

    pub fn shutdown(&self) {
        self.compact();
    }

    pub fn frecency_score(&self, id: &str) -> u32 {
        let data = self.data.lock();
        let Some(entry) = data.get(id) else { return 0 };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let age_secs = now.saturating_sub(entry.last_used);
        entry.count * decay_factor(age_secs)
    }

    pub fn top_ids(&self, n: usize) -> Vec<String> {
        struct ScoredId {
            id: String,
            score: u32,
        }

        let data = self.data.lock();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut scored: Vec<ScoredId> = data
            .iter()
            .map(|(id, entry)| {
                let age_secs = now.saturating_sub(entry.last_used);
                ScoredId {
                    id: id.clone(),
                    score: entry.count * decay_factor(age_secs),
                }
            })
            .collect();

        if scored.len() <= n {
            scored.sort_by_key(|s| std::cmp::Reverse(s.score));
            return scored.into_iter().map(|s| s.id).collect();
        }

        scored.select_nth_unstable_by_key(n, |s| std::cmp::Reverse(s.score));
        scored.truncate(n);
        scored.sort_by_key(|s| std::cmp::Reverse(s.score));
        scored.into_iter().map(|s| s.id).collect()
    }
}

fn replay_log_into(log_path: &PathBuf, data: &mut HashMap<String, Entry>) -> usize {
    let file = match File::open(log_path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let mut count = 0;
    for line in BufReader::new(file).lines() {
        let Ok(line) = line else { continue };
        let Ok(record) = serde_json::from_str::<LogRecord>(&line) else {
            continue;
        };
        let entry = data.entry(record.id).or_insert(Entry {
            count: 0,
            last_used: 0,
        });
        entry.count += 1;
        entry.last_used = record.ts;
        count += 1;
    }
    count
}

const ONE_HOUR: u64 = 3600;
const ONE_DAY: u64 = 86400;
const DECAY_RECENT: u32 = 4;
const DECAY_TODAY: u32 = 2;
const DECAY_OLD: u32 = 1;

fn decay_factor(age_secs: u64) -> u32 {
    if age_secs < ONE_HOUR {
        DECAY_RECENT
    } else if age_secs < ONE_DAY {
        DECAY_TODAY
    } else {
        DECAY_OLD
    }
}
