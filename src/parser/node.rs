use super::bracket::parse_bracket;
use super::bracket_extension::parse_bracket_extension;
use super::code_inline::parse_inline_code;
use super::deco::parse_deco;
use super::hashtag::parse_hashtag;
use super::math_inline::parse_math_inline;
use super::text::parse_text;
use crate::CosyParserExtension;
use crate::ast::Node;
use winnow::Result as PResult;
use winnow::combinator::{alt, repeat};
use winnow::prelude::*;

pub fn parse_nodes<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Vec<Node<E::Output>>>
where
    E: CosyParserExtension,
{
    repeat(
        0..,
        alt((
            parse_inline_code,
            parse_math_inline,
            parse_bracket_extension(extension),
            parse_deco(extension),
            parse_bracket(extension),
            parse_hashtag,
            parse_text,
        )),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::parse_nodes;
    use crate::ast::{Link, Node};

    #[test]
    fn test_parse_math_node() {
        let mut input = "[$ y=a^2 + b^2] and more text";
        let result = parse_nodes(&mut input, &()).unwrap();

        let expected = vec![
            Node::Math("y=a^2 + b^2".to_string()),
            Node::Text(" and more text".to_string()),
        ];

        assert_eq!(result, expected);
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_hashtag_node() {
        let mut input = "テキスト #タグ 続き";
        let result = parse_nodes(&mut input, &()).unwrap();
        let expected = vec![
            Node::Text("テキスト ".to_string()),
            Node::Hashtag("タグ".to_string()),
            Node::Text(" 続き".to_string()),
        ];
        assert_eq!(result, expected);
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_link_node() {
        let mut input = "[Link text http://example.com] and more text";
        let result = parse_nodes(&mut input, &()).unwrap();

        let expected = vec![
            Node::Link(Link::WithLabel {
                label: vec![Node::Text("Link text".to_string())],
                href: "http://example.com".to_string(),
            }),
            Node::Text(" and more text".to_string()),
        ];

        assert_eq!(result, expected);
        assert_eq!(input, "");
    }
}
