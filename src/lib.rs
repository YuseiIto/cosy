#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! A parser for [Cosense] (formerly Scrapbox) markup syntax.
//!
//! cosy converts Cosense/Scrapbox markup text into a typed Abstract Syntax Tree (AST).
//! It is built with the [`winnow`] parser combinator library and supports user-defined
//! syntax extensions via the [`CosyParserExtension`] trait.
//!
//! [Cosense]: https://cosen.se/
//! [`winnow`]: https://docs.rs/winnow
//!
//! # Quick start
//!
//! ```rust
//! let doc = cosy::parse("Hello [* world]!", &()).unwrap();
//!
//! // With `&()` as the extension, `E::Output` is `()`,
//! // so `doc` is a `Document<()>` (i.e. `Vec<Block<()>>`).
//! assert_eq!(doc.len(), 1);
//! ```
//!
//! # AST structure
//!
//! A [`ast::Document`] is a `Vec<`[`ast::Block`]`<T>>`. Each block has an
//! `indent` level and a [`ast::BlockContent`] variant:
//!
//! - [`ast::BlockContent::Line`] — a normal text line, containing [`ast::Node`]s
//! - [`ast::BlockContent::CodeBlock`] — a fenced code block (`code:filename`)
//! - [`ast::BlockContent::Table`] — a table block (`table:name`)
//! - [`ast::BlockContent::Quote`] — a quoted line (`> ...`)
//! - [`ast::BlockContent::Helpfeel`] — a Helpfeel search query (`? ...`)
//! - [`ast::BlockContent::CommandLine`] — a shell command line (`$ ...`)
//! - [`ast::BlockContent::Custom`] — a user-defined extension block
//!
//! Inline [`ast::Node`] variants include [`ast::Node::Text`], [`ast::Node::Link`],
//! [`ast::Node::Image`], [`ast::Node::LinkedImage`], [`ast::Node::Icon`],
//! [`ast::Node::InlineCode`], [`ast::Node::Math`], [`ast::Node::Hashtag`],
//! [`ast::Node::Decoration`], [`ast::Node::Strong`], [`ast::Node::Coordinate`],
//! and [`ast::Node::Custom`].
//!
//! # Extending the parser
//!
//! Implement [`CosyParserExtension`] to add your own bracket or block syntax:
//!
//! ```rust
//! use cosy::CosyParserExtension;
//!
//! #[derive(Debug, PartialEq)]
//! enum MySyntax { Highlight(String) }
//!
//! struct MyExt;
//! impl CosyParserExtension for MyExt {
//!     type Output = MySyntax;
//!     fn parse_bracket(&self, content: &str) -> Option<MySyntax> {
//!         content.strip_prefix("! ").map(|s| MySyntax::Highlight(s.to_string()))
//!     }
//! }
//!
//! let doc = cosy::parse("[! important]", &MyExt).unwrap();
//! ```
//!
//! # Feature flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `serde` | Derive `serde::Serialize` and `serde::Deserialize` on all AST types |

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
/// allowing users to define custom syntax for brackets and blocks.
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
