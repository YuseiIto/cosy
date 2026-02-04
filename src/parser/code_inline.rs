use crate::ast::Node;
use crate::tokens::BACKTICK;
use winnow::Result as PResult;
use winnow::combinator::delimited;
use winnow::prelude::*;
use winnow::token::take_until;

// ` code `
pub fn parse_inline_code<'s, T>(input: &mut &'s str) -> PResult<Node<T>> {
    // Basic implementation: `...`
    let content = delimited(BACKTICK, take_until(0.., BACKTICK), BACKTICK).parse_next(input)?;
    Ok(Node::InlineCode(content.to_string()))
}

#[test]
fn test_parse_inline_code() {
    let mut input = "`inline code` and more text";
    let result: Node<()> = parse_inline_code(&mut input).unwrap();
    assert_eq!(result, Node::InlineCode("inline code".to_string()));
    assert_eq!(input, " and more text");
}
