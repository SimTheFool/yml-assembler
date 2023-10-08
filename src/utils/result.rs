#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("{0}")]
    FileSystem(String),
    #[error("{0}")]
    ParseYml(String),
    #[error("{0}")]
    ValidateYml(String),
    #[error("{0}")]
    ApplyFormula(String),
}

impl AppError {
    pub fn other<T>(e: T) -> AppError
    where
        T: Into<anyhow::Error>,
    {
        AppError::Other(e.into())
    }

    pub fn formula<T>(e: T) -> AppError
    where
        T: Into<String>,
    {
        AppError::ApplyFormula(e.into())
    }
}

pub type AppResult<T> = Result<T, AppError>;
