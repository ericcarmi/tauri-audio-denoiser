use thiserror::Error;

#[derive(Error, Debug)]
pub enum DenoiserError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Cpal(#[from] cpal::DevicesError),

    #[error(transparent)]
    CpalConfig(#[from] cpal::DefaultStreamConfigError),

    #[error(transparent)]
    CpalBuildStream(#[from] cpal::BuildStreamError),

    #[error("other error message")]
    Other(String),
}

pub type DenoiserResult<T, E = DenoiserError> = Result<T, E>;

impl serde::Serialize for DenoiserError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
