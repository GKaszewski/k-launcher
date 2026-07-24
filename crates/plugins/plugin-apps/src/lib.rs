mod cache;
pub mod frecency;
#[cfg(target_os = "linux")]
pub mod linux;
mod plugin;
mod scoring;
mod types;

pub use cache::{CachedEntry, build_entries, load_from_path, save_to_path};
pub use plugin::*;
pub use scoring::{humanize_category, new_matcher, parse_pattern, score_match};
pub use types::*;
