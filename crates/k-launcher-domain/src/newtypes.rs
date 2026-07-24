#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ResultId(String);

impl ResultId {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        debug_assert!(!id.is_empty(), "ResultId must not be empty");
        Self(id)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResultTitle(String);

impl ResultTitle {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        debug_assert!(!title.is_empty(), "ResultTitle must not be empty");
        Self(title)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Score(u32);

impl Score {
    pub const MAX: Self = Self(u32::MAX);

    pub fn new(value: u32) -> Self {
        Self(value)
    }
    pub fn value(self) -> u32 {
        self.0
    }
    pub fn saturating_add(self, other: u32) -> Self {
        Self(self.0.saturating_add(other))
    }
}
