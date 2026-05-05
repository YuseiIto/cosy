use crate::ast::Node;
use crate::tokens::ICON_SUFFIX;
use winnow::Result as PResult;
use winnow::error::ContextError;

pub(super) fn parse_icon<T>(input: &mut &str) -> PResult<Node<T>> {
    // [name.icon] or [name.icon*N] (N > 0)
    let content = *input;

    let (icon_part, count) = content
        .rsplit_once('*')
        .filter(|(before, _)| before.ends_with(ICON_SUFFIX))
        .and_then(|(before, after)| {
            let n = after.parse::<usize>().ok()?;
            (n > 0).then_some((before, n))
        })
        .unwrap_or((content, 1));

    let name = icon_part
        .strip_suffix(ICON_SUFFIX)
        .filter(|name| !name.is_empty())
        .ok_or_else(ContextError::new)?;

    *input = "";
    Ok(Node::Icon {
        name: name.to_string(),
        count,
    })
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
