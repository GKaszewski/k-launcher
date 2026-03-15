#[cfg(unix)]
pub fn home_dir() -> Option<String> {
    std::env::var("HOME").ok()
}

#[cfg(windows)]
pub fn home_dir() -> Option<String> {
    std::env::var("USERPROFILE").ok()
}
