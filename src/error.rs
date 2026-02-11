use thiserror::Error;

/// Custom error types for the LLM request and deserialization process.
#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Failed to send request: {0}")]
    ChatRequest(#[from] genai::Error),

    #[error(
        "Retry limit exceeded, the following errors occurred when trying to send requests: {0:?}"
    )]
    RetryLimitExceeded(Vec<RequestError>),

    #[error("XML deserialization error: {0}")]
    XmlDeserialization(#[from] quick_xml::DeError),

    #[error("XML extraction error: {0}")]
    XmlExtraction(String),
}
