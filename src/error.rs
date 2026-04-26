//! Error types for the cosy parser.

/// An error that occurred during parsing.
///
/// This type is returned when the input cannot be parsed as valid
/// Cosense/Scrapbox markup.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("parse error: {message}")]
pub struct ParseError {
    /// A human-readable description of the parse failure.
    pub message: String,
}
