pub mod rtsp;

use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::StreamIngester;

/// Factory function to create the appropriate protocol adapter
pub fn create_stream_ingester(
    protocol: &str,
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<Box<dyn StreamIngester>, DomainError> {
    match protocol {
        "rtsp" => Ok(Box::new(rtsp::RtspIngester::new(url, username, password))),
        other => Err(DomainError::Validation(format!(
            "Unsupported protocol: {}. Supported: rtsp",
            other
        ))),
    }
}
