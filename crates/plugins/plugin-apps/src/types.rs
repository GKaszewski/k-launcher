// --- Domain newtypes ---

#[derive(Debug, Clone)]
pub struct AppName(String);

impl AppName {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ExecCommand(String);

impl ExecCommand {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct IconPath(String);

impl IconPath {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// --- Desktop entry ---

pub struct DesktopEntry {
    pub name: AppName,
    pub exec: ExecCommand,
    pub icon: Option<IconPath>,
    pub category: Option<String>,
    pub keywords: Vec<String>,
}

// --- Swappable source trait (Application layer principle) ---

pub trait DesktopEntrySource: Send + Sync {
    fn entries(&self) -> Vec<DesktopEntry>;
}
