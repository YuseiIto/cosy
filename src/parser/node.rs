use super::bracket::parse_bracket;
use super::bracket_extension::parse_bracket_extension;
use super::code_inline::parse_inline_code;
use super::deco::parse_deco;
use super::hashtag::parse_hashtag;
use super::strong::parse_strong;
use super::text::parse_text;
use crate::CosyParserExtension;
use crate::ast::Node;
use winnow::Result as PResult;
use winnow::combinator::{alt, repeat};
use winnow::prelude::*;
use winnow::token::any;

pub fn parse_nodes<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Vec<Node<E::Output>>>
where
    E: CosyParserExtension,
{
    let nodes: Vec<Node<E::Output>> = repeat(
        0..,
        alt((
            parse_inline_code,
            parse_bracket_extension(extension),
            parse_strong(extension),
            parse_deco(extension),
            parse_bracket(extension),
            parse_hashtag,
            parse_text,
            parse_one_char_text,
        )),
    )
    .parse_next(input)?;
    Ok(coalesce_text(nodes))
}

/// Like `parse_nodes` but without `parse_deco` — used inside decoration content
/// so that `[* [** text]]` treats the inner bracket as a page link, not a nested decoration.
pub fn parse_nodes_no_deco<'s, E>(
    input: &mut &'s str,
    extension: &'s E,
) -> PResult<Vec<Node<E::Output>>>
where
    E: CosyParserExtension,
{
    let nodes: Vec<Node<E::Output>> = repeat(
        0..,
        alt((
            parse_inline_code,
            parse_bracket_extension(extension),
            parse_strong(extension),
            parse_bracket(extension),
            parse_hashtag,
            parse_text,
            parse_one_char_text,
        )),
    )
    .parse_next(input)?;
    Ok(coalesce_text(nodes))
}

/// Last-resort branch for `parse_nodes`: preserves a stray trigger char (`[`,
/// `` ` ``, `#`) as plain text instead of letting `repeat(0..)` drop it.
fn parse_one_char_text<T>(input: &mut &str) -> PResult<Node<T>> {
    let c = any.parse_next(input)?;
    Ok(Node::Text(c.to_string()))
}

/// Undoes the per-char fragmentation introduced by `parse_one_char_text`.
fn coalesce_text<T>(nodes: Vec<Node<T>>) -> Vec<Node<T>> {
    let mut out: Vec<Node<T>> = Vec::with_capacity(nodes.len());
    for node in nodes {
        match (out.last_mut(), node) {
            (Some(Node::Text(prev)), Node::Text(curr)) => prev.push_str(&curr),
            (_, n) => out.push(n),
        }
    }
    out
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

    #[test]
    fn parse_lone_hash_preserved() {
        let mut input = "# alone";
        let result = parse_nodes(&mut input, &()).unwrap();
        assert_eq!(result, vec![Node::Text("# alone".to_string())]);
        assert_eq!(input, "");
    }

    #[test]
    fn parse_unclosed_bracket_preserved() {
        let mut input = "before [unclosed";
        let result = parse_nodes(&mut input, &()).unwrap();
        assert_eq!(result, vec![Node::Text("before [unclosed".to_string())]);
        assert_eq!(input, "");
    }

    #[test]
    fn parse_unclosed_backtick_preserved() {
        let mut input = "a `unclosed";
        let result = parse_nodes(&mut input, &()).unwrap();
        assert_eq!(result, vec![Node::Text("a `unclosed".to_string())]);
        assert_eq!(input, "");
    }

    #[test]
    fn parse_lone_hash_then_valid_hashtag() {
        let mut input = "# bare and #tag end";
        let result = parse_nodes(&mut input, &()).unwrap();
        assert_eq!(
            result,
            vec![
                Node::Text("# bare and ".to_string()),
                Node::Hashtag("tag".to_string()),
                Node::Text(" end".to_string()),
            ]
        );
        assert_eq!(input, "");
    }
}
