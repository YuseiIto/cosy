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
mod strong;
mod table;
mod text;

use block::parse_block;

/// Internal winnow-based parser. Use `crate::parse()` for the public API.
pub(crate) fn parse_inner<'s, E>(
    input: &mut &'s str,
    extension: &'s E,
) -> PResult<Document<E::Output>>
where
    E: CosyParserExtension,
{
    repeat(0.., |i: &mut &'s str| parse_block(i, extension)).parse_next(input)
}
