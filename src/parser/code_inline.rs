use crate::ast::Node;
use winnow::Result as PResult;
use winnow::combinator::delimited;
use winnow::prelude::*;
use winnow::token::take_until;

// ` code `
pub fn parse_inline_code<'s, T>(input: &mut &'s str) -> PResult<Node<T>> {
    // Basic implementation: `...`
    let content = delimited('`', take_until(0.., '`'), '`').parse_next(input)?;
    Ok(Node::InlineCode(content.to_string()))
}
