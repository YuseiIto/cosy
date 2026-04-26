//! Core parsing features.
//!
//! This module contains the main entry point for parsing text into an Abstract Syntax Tree (AST).
//! It handles the breakdown of the input string into blocks, such as lines of text,
//! code blocks, tables, and quotes.

use crate::CosyParserExtension;
use crate::ast::Document;
use winnow::Result as PResult;
use winnow::combinator::repeat;
use winnow::prelude::*;

mod block;
mod bracket;
mod bracket_content;
mod bracket_extension;
mod code;
mod code_inline;
mod commandline;
mod deco;
mod hashtag;
mod helpfeel;
mod line;
mod node;
mod quote;
mod table;
mod text;

use block::parse_block;

/// Parses an input string into a `Document` AST node.
///
/// This is the main primary point of the parser. It processes the
/// input string by repeatedly applying the block parser until the entire input is consumed.
/// The result is a `Document` containing a vector of `Block` nodes.
///
/// The function supports extensibility through the `CosyParserExtension` trait, allowing
/// users to define custom syntax for brackets and blocks.
///
/// # Arguments
///
/// * `input` - A mutable reference to the input string slice. The parser advances this slice
///             as it consumes the input.
/// * `extension` - A reference to an implementation of `CosyParserExtension`. Use `&()` if
///                 no custom extensions are needed.
///
/// # Returns
///
/// Returns a `PResult<Document<E::Output>>`. On success, it contains the parsed `Document`.
/// On failure, it returns a parsing error.
///
/// # Examples
///
/// ## Basic Usage
///
/// Parsing simple text with standard syntax:
///
/// ```rust
/// use cosy;
///
/// let mut input = "[* Bold text] and [https://example.com Link]";
/// let result = cosy::parse(&mut input, &());
///
/// assert!(result.is_ok());
///
/// ```
pub fn parse<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Document<E::Output>>
where
    E: CosyParserExtension,
{
    repeat(0.., |i: &mut &'s str| parse_block(i, extension)).parse_next(input)
}
