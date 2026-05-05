use super::bracket_content::take_bracket_content;
use crate::CosyParserExtension;
use crate::ast::Node;
use crate::tokens::{ICON_SUFFIX, LBRACKET, RBRACKET};
use crate::url::{UrlKind, infer_url};
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

        let nodes = match content.strip_suffix(ICON_SUFFIX) {
            Some("") => return Err(ContextError::new()),
            Some(name) => vec![Node::Icon {
                name: name.to_string(),
                count: 1,
            }],
            None => match infer_url(content) {
                Some((_, UrlKind::Image)) => vec![Node::Image(content.to_string())],
                Some((_, UrlKind::Other)) => {
                    vec![Node::Link(crate::ast::Link::Url(content.to_string()))]
                }
                None => parse_nodes_no_deco(&mut content, extension)?,
            },
        };
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
    fn test_parse_strong_image() {
        let mut input = "[[https://example.com/photo.png]]";
        let result: Node<()> = parse_strong(&()).parse_next(&mut input).unwrap();
        assert_eq!(
            result,
            Node::Strong(vec![Node::Image(
                "https://example.com/photo.png".to_string()
            )])
        );
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_strong_non_image_url_is_link() {
        let mut input = "[[https://example.com/page]]";
        let result: Node<()> = parse_strong(&()).parse_next(&mut input).unwrap();
        assert_eq!(
            result,
            Node::Strong(vec![Node::Link(crate::ast::Link::Url(
                "https://example.com/page".to_string()
            ))])
        );
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_strong_icon() {
        let mut input = "[[user.icon]]";
        let result: Node<()> = parse_strong(&()).parse_next(&mut input).unwrap();
        assert_eq!(
            result,
            Node::Strong(vec![Node::Icon {
                name: "user".to_string(),
                count: 1,
            }])
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
