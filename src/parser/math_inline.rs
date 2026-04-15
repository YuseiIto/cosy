use super::bracket_content::take_bracket_content;
use crate::ast::Node;
use crate::tokens::RBRACKET;
use winnow::Result as PResult;
use winnow::combinator::delimited;
use winnow::prelude::*;

// [$ math]
pub fn parse_math_inline<'s, T>(input: &mut &'s str) -> PResult<Node<T>> {
    let content = delimited("[$", take_bracket_content, RBRACKET).parse_next(input)?;
    // Math
    let math_content = &content[1..];
    Ok(Node::Math(math_content.trim().to_string()))
}

#[test]
fn test_parse_inline_code() {
    let mut input = "[$ y=a^2 + b^2] and more text";
    let result: Node<()> = parse_math_inline(&mut input).unwrap();
    assert_eq!(result, Node::Math("y=a^2 + b^2".to_string()));
    assert_eq!(input, " and more text");
}
