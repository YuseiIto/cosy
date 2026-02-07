use crate::CosyParserExtension;
use crate::ast::Node;
use crate::tokens::{DECO_CHARS, LBRACKET, RBRACKET};
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{take_until, take_while};

use super::node::parse_nodes;

pub fn parse_deco<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    E: CosyParserExtension,
{
    move |input: &mut &'i str| {
        let mut content: &str =
            delimited(LBRACKET, take_until(0.., RBRACKET), RBRACKET).parse_next(input)?;
        let decos = take_while(1.., |c| is_deco_char(c)).parse_next(&mut content)?;

        // Skip spaces after decoration
        let _ = take_while(1.., AsChar::is_space).parse_next(&mut content)?;

        let nodes = parse_nodes(&mut content, extension)?;
        Ok(Node::Decoration {
            decos: decos.to_string(),
            nodes: nodes,
        })
    }
}

fn is_deco_char(c: char) -> bool {
    DECO_CHARS.contains(c)
}

#[cfg(test)]
mod tests {
    use super::parse_deco;
    use crate::ast::{Link, Node};
    use winnow::prelude::*;

    #[test]
    fn test_parse_deco() {
        let mut input = "[** Hello World] and more text";
        let result: Node<()> = parse_deco(&()).parse_next(&mut input).unwrap();
        let expected = Node::Decoration {
            decos: "**".to_string(),
            nodes: vec![Node::Text("Hello World".to_string())],
        };
        assert_eq!(result, expected);
        assert_eq!(input, " and more text");
    }

    #[test]
    fn test_parse_deco_with_links() {
        let mut input = "[* Click [here](http://example.com)] and more text";
        let result: Node<()> = parse_deco(&()).parse_next(&mut input).unwrap();
        let expected = Node::Decoration {
            decos: "*".to_string(),
            nodes: vec![
                Node::Text("Click ".to_string()),
                Node::Link(Link::WithLabel {
                    href: "http://example.com".to_string(),
                    label: vec![Node::Text("here".to_string())],
                }),
            ],
        };
        assert_eq!(result, expected);
        assert_eq!(input, " and more text");
    }
}
