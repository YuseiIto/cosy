//! Error types for the cosy parser.

/// An error that occurred during parsing.
///
/// This type is returned when the input cannot be parsed as valid
/// Cosense/Scrapbox markup.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("parse error: {message}")]
#[non_exhaustive]
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub(crate) fn new(message: String) -> Self {
        Self { message }
    }

    /// Returns a human-readable description of the parse failure.
    pub fn message(&self) -> &str {
        &self.message
    }
}
