use super::bracket_content::take_bracket_content;
use crate::CosyParserExtension;
use crate::ast::Node;
use crate::tokens::{LBRACKET, RBRACKET};
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;

use super::node::parse_nodes_no_deco;

/// Parses `[[...]]` Strong notation.
///
/// Returns `Node::Strong` containing the recursively parsed inner nodes.
/// Fails immediately if the input does not start with `[[`, so that
/// `alt()` can fall through to the standard bracket parser.
pub fn parse_strong<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    E: CosyParserExtension,
{
    move |input: &mut &'i str| {
        if !input.starts_with("[[") {
            return Err(ContextError::new());
        }

        // Outer delimited([, ..., ]) wraps the inner delimited([, content, ])
        // For [[text]]:
        //   outer [ consumed → remaining [text]]
        //   inner delimited extracts "text" and consumes [text] → remaining ]
        //   outer ] consumed → done
        let mut content: &str = delimited(
            LBRACKET,
            delimited(LBRACKET, take_bracket_content, RBRACKET),
            RBRACKET,
        )
        .parse_next(input)?;

        let nodes = parse_nodes_no_deco(&mut content, extension)?;
        Ok(Node::Strong(nodes))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_strong;
    use crate::ast::{Link, Node};
    use winnow::prelude::*;

    #[test]
    fn test_parse_strong_basic() {
        let mut input = "[[Bold text]]";
        let result: Node<()> = parse_strong(&()).parse_next(&mut input).unwrap();
        assert_eq!(
            result,
            Node::Strong(vec![Node::Text("Bold text".to_string())])
        );
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_strong_multiple_words() {
        let mut input = "[[Multiple words here]] rest";
        let result: Node<()> = parse_strong(&()).parse_next(&mut input).unwrap();
        assert_eq!(
            result,
            Node::Strong(vec![Node::Text("Multiple words here".to_string())])
        );
        assert_eq!(input, " rest");
    }

    #[test]
    fn test_parse_strong_with_nested_link() {
        let mut input = "[[Link inside [Page] here]]";
        let result: Node<()> = parse_strong(&()).parse_next(&mut input).unwrap();
        assert_eq!(
            result,
            Node::Strong(vec![
                Node::Text("Link inside ".to_string()),
                Node::Link(Link::Page("Page".to_string())),
                Node::Text(" here".to_string()),
            ])
        );
        assert_eq!(input, "");
    }

    #[test]
    fn test_single_bracket_not_strong() {
        let mut input = "[text]";
        let result = parse_strong::<()>(&()).parse_next(&mut input);
        assert!(result.is_err());
        // Input must be unchanged
        assert_eq!(input, "[text]");
    }
}
