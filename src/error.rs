use thiserror::Error;

pub type Result<T> = std::result::Result<T, AeroBaseError>;

#[derive(Error, Debug)]
pub enum AeroBaseError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Device fingerprint error: {0}")]
    DeviceFingerprint(String),

    #[error("Spatial query error: {0}")]
    SpatialQuery(String),

    #[error("Flight planning error: {0}")]
    FlightPlanning(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<r2d2::Error> for AeroBaseError {
    fn from(err: r2d2::Error) -> Self {
        AeroBaseError::Pool(err.to_string())
    }
}
