use crate::ast::Node;
use crate::tokens::LBRACKET;
use winnow::prelude::*;
use winnow::token::take_till;
use winnow::Result as PResult;

pub fn parse_text<'s, T>(input: &mut &'s str) -> PResult<Node<T>> {
    let text = take_till(1.., |c| c == LBRACKET).parse_next(input)?;
    Ok(Node::Text(text.to_string()))
}

#[test]
fn test_parse_text() {
    let mut input = "これはテストです。[リンク]";
    let result: Node<()> = parse_text(&mut input).unwrap();
    assert_eq!(result, Node::Text("これはテストです。".to_string()));
    assert_eq!(input, "[リンク]");
}
