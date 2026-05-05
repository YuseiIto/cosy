#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod ast;
pub mod error;
mod extension;
mod parser;
mod tokens;
mod url;

pub use ::url::Url;
pub use error::ParseError;
pub use extension::CosyParserExtension;

/// Parses an input string into a [`ast::Document`] AST.
///
/// This is the main entry point of the parser. It processes the entire input
/// string and returns a [`ast::Document`] (a thin newtype around
/// `Vec<Block<E::Output>>`).
///
/// The function supports extensibility through the [`CosyParserExtension`] trait,
/// allowing users to define custom syntax for brackets.
///
/// # Arguments
///
/// * `input` - The Cosense/Scrapbox markup string to parse.
/// * `extension` - An implementation of [`CosyParserExtension`]. Use `&()` if
///   no custom extensions are needed. In that case `E::Output` is `()`,
///   producing a `Document<()>`.
///
/// # Errors
///
/// Returns a [`ParseError`] if the input cannot be parsed. In practice the
/// built-in parser is infallible for any UTF-8 input (unknown syntax is
/// treated as plain text), so errors only arise from custom extensions.
///
/// # Examples
///
/// ```rust
/// use cosy;
///
/// let result = cosy::parse("[* Bold text] and [https://example.com Link]", &());
/// assert!(result.is_ok());
/// ```
pub fn parse<E>(input: &str, extension: &E) -> Result<ast::Document<E::Output>, ParseError>
where
    E: CosyParserExtension,
{
    let mut s = input;
    parser::parse_inner(&mut s, extension).map_err(|e| ParseError::new(e.to_string()))
}
