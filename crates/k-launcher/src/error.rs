use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("UI error: {0}")]
    Ui(String),
}
