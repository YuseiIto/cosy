use crate::ast::Node;
use crate::tokens::LBRACKET;
use winnow::prelude::*;
use winnow::token::take_till;
use winnow::Result as PResult;

pub fn parse_text<'s, T>(input: &mut &'s str) -> PResult<Node<T>> {
    // Stop at [, $, `
    // Also stop at \n because blocks are line-based generally, but parse_nodes handles lines.
    // parse_nodes calls this.
    // We should probably consume at least one char to avoid infinite loop if we get stuck,
    // but take_till(1..) ensures we take at least one char or fail.
    // If next char is [, $, or `, take_till will fail (return 0 len match if 1..).
    // This allows alt() in parse_nodes to try other parsers.
    
    let text = take_till(1.., |c| c == LBRACKET || c == '$' || c == '`').parse_next(input)?;
    Ok(Node::Text(text.to_string()))
}

#[test]
fn test_parse_text() {
    let mut input = "これはテストです。[リンク]";
    let result: Node<()> = parse_text(&mut input).unwrap();
    assert_eq!(result, Node::Text("これはテストです。".to_string()));
    assert_eq!(input, "[リンク]");
}