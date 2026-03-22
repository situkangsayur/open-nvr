use serde::{Deserialize, Serialize};
use crate::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUrl(String);

impl StreamUrl {
    pub fn new(url: &str) -> Result<Self, DomainError> {
        if url.is_empty() {
            return Err(DomainError::Validation("Stream URL cannot be empty".into()));
        }
        // Basic validation - must start with a known scheme
        let valid_schemes = ["rtsp://", "rtsps://", "http://", "https://", "rtmp://"];
        if !valid_schemes.iter().any(|s| url.starts_with(s)) {
            return Err(DomainError::Validation(format!(
                "Invalid stream URL scheme. Must start with one of: {:?}", valid_schemes
            )));
        }
        Ok(Self(url.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for StreamUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
