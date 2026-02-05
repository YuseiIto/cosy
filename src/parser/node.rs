use super::code_inline::parse_inline_code;
use crate::CosyParserExtension;
use crate::ast::Node;
use winnow::Result as PResult;
use winnow::combinator::{alt, repeat};
use winnow::prelude::*;

use super::bracket::parse_bracket;
use super::text::parse_text;

pub fn parse_nodes<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Vec<Node<E::Output>>>
where
    E: CosyParserExtension,
{
    repeat(
        0..,
        alt((parse_inline_code, parse_bracket(extension), parse_text)),
    )
    .parse_next(input)
}
