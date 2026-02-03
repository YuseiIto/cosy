use crate::ast::Node;
use crate::ExtensionParser;
use winnow::combinator::{alt, repeat, delimited};
use winnow::token::take_until;
use winnow::prelude::*;
use winnow::Result as PResult;

use super::bracket::parse_bracket;
use super::text::parse_text;

pub fn parse_nodes<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Vec<Node<E::Output>>>
where
    E: ExtensionParser,
{
    repeat(
        0..,
        alt((
            parse_math,
            parse_inline_code,
            parse_bracket(extension),
            parse_text,
        )),
    )
    .parse_next(input)
}

// $ math $
fn parse_math<'s, T>(input: &mut &'s str) -> PResult<Node<T>> {
    // Basic implementation: $...$
    // Not handling escaped $ for now
    let content = delimited('$', take_until(0.., '$'), '$').parse_next(input)?;
    Ok(Node::Math(content.to_string()))
}

// ` code `
fn parse_inline_code<'s, T>(input: &mut &'s str) -> PResult<Node<T>> {
    // Basic implementation: `...`
    let content = delimited('`', take_until(0.., '`'), '`').parse_next(input)?;
    Ok(Node::InlineCode(content.to_string()))
}