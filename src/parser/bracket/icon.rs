use crate::ast::Node;
use crate::tokens::ICON_SUFFIX;
use winnow::Result as PResult;
use winnow::ascii::dec_uint;
use winnow::combinator::{eof, opt, preceded, terminated};
use winnow::prelude::*;
use winnow::token::take_until;

pub(super) fn parse_icon<T>(input: &mut &str) -> PResult<Node<T>> {
    // <name>.icon or <name>.icon*N (N > 0); name must be non-empty.
    terminated(
        (
            take_until(1.., ICON_SUFFIX),
            ICON_SUFFIX,
            opt(preceded('*', dec_uint::<_, usize, _>).verify(|n| *n > 0)),
        ),
        eof,
    )
    .map(|(name, _, count): (&str, _, Option<usize>)| Node::Icon {
        name: name.to_string(),
        count: count.unwrap_or(1),
    })
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::super::parse_bracket;
    use crate::ast::{Link, Node};
    use winnow::Parser;

    fn parse(input: &str) -> Node<()> {
        let mut s = input;
        parse_bracket(&()).parse_next(&mut s).unwrap()
    }

    #[test]
    fn test_icon_simple() {
        let node = parse("[user.icon]");
        assert_eq!(
            node,
            Node::Icon {
                name: "user".to_string(),
                count: 1
            }
        );
    }

    #[test]
    fn test_icon_repeat() {
        let node = parse("[user.icon*3]");
        assert_eq!(
            node,
            Node::Icon {
                name: "user".to_string(),
                count: 3
            }
        );
    }

    #[test]
    fn test_icon_repeat_one() {
        let node = parse("[user.icon*1]");
        assert_eq!(
            node,
            Node::Icon {
                name: "user".to_string(),
                count: 1
            }
        );
    }

    #[test]
    fn test_icon_invalid_count_zero() {
        let node = parse("[user.icon*0]");
        assert_eq!(node, Node::Link(Link::Page("user.icon*0".to_string())));
    }

    #[test]
    fn test_icon_invalid_count_str() {
        let node = parse("[user.icon*abc]");
        assert_eq!(node, Node::Link(Link::Page("user.icon*abc".to_string())));
    }

    #[test]
    fn test_icon_name_with_spaces() {
        // [user name.icon] → Icon { name: "user name", count: 1 }
        let node = parse("[user name.icon]");
        assert_eq!(
            node,
            Node::Icon {
                name: "user name".to_string(),
                count: 1,
            }
        );
    }

    #[test]
    fn test_icon_empty_name_is_page_link() {
        // [.icon] has empty name — should fall through to Link::Page
        let node = parse("[.icon]");
        assert_eq!(node, Node::Link(Link::Page(".icon".to_string())));
    }
}
