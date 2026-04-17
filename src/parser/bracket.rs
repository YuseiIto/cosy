use super::bracket_content::take_bracket_content;
use crate::CosyParserExtension;
use crate::ast::Link;
use crate::ast::Node;
use crate::tokens::{ICON_SUFFIX, LBRACKET, RBRACKET};
use crate::url::{UrlKind, infer_url_kind, is_url};
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;

use super::node::parse_nodes;

pub fn parse_bracket<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    E: CosyParserExtension,
{
    move |input: &mut &'i str| {
        let content: &str =
            delimited(LBRACKET, take_bracket_content, RBRACKET).parse_next(input)?;

        // 2. Icon: [name.icon] or [name.icon*3]
        if content.ends_with(ICON_SUFFIX) {
            // Simple icon
            let name = content.trim_end_matches(ICON_SUFFIX);
            return Ok(Node::Icon {
                name: name.to_string(),
                count: 1,
            });
        }
        if let Some((name_part, count_str)) = content.rsplit_once('*')
            && name_part.ends_with(ICON_SUFFIX)
            && let Ok(count) = count_str.parse::<usize>()
            && count > 0
        {
            let name = name_part.trim_end_matches(ICON_SUFFIX);
            return Ok(Node::Icon {
                name: name.to_string(),
                count,
            });
        }

        // 3. Cross-project link: [/project], [/project/], or [/project/page]
        if let Some(rest) = content.strip_prefix('/') {
            if let Some((project, page)) = rest.split_once('/') {
                if !project.is_empty() {
                    if page.is_empty() {
                        return Ok(Node::Link(Link::Project(project.to_string())));
                    } else {
                        return Ok(Node::Link(Link::ProjectPage {
                            project: project.to_string(),
                            page: page.to_string(),
                        }));
                    }
                }
            } else if !rest.is_empty() {
                return Ok(Node::Link(Link::Project(rest.to_string())));
            }
        }

        // 4. Links (recurse on label)
        // Split by space

        if let Some((left, right)) = content.rsplit_once(' ') {
            let left = left.trim();
            let right = right.trim();

            if is_url(left) {
                // [url label]
                let mut label_input = right;
                let nodes = parse_nodes(&mut label_input, extension)?;
                return Ok(Node::Link(Link::WithLabel {
                    href: left.to_string(),
                    label: nodes,
                }));
            } else if is_url(right) {
                // [label url]
                let mut label_input = left;
                let nodes = parse_nodes(&mut label_input, extension)?;
                return Ok(Node::Link(Link::WithLabel {
                    href: right.to_string(),
                    label: nodes,
                }));
            } else {
                // [Page Name] - Space inside page name
                return Ok(Node::Link(Link::Page(content.to_string())));
            }
        }

        // 4. Simple content (Image, URL, Page)
        match infer_url_kind(content) {
            Some(UrlKind::Image) => Ok(Node::Image(content.to_string())),
            Some(UrlKind::Other) => Ok(Node::Link(Link::Url(content.to_string()))),
            None => Ok(Node::Link(Link::Page(content.to_string()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_bracket;
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
    fn test_project_page_basic() {
        let node = parse("[/project/page]");
        assert_eq!(
            node,
            Node::Link(Link::ProjectPage {
                project: "project".to_string(),
                page: "page".to_string(),
            })
        );
    }

    #[test]
    fn test_project_page_with_spaces() {
        let node = parse("[/project/page with spaces]");
        assert_eq!(
            node,
            Node::Link(Link::ProjectPage {
                project: "project".to_string(),
                page: "page with spaces".to_string(),
            })
        );
    }

    #[test]
    fn test_project_only_link() {
        // [/project] → Link::Project
        let node = parse("[/project]");
        assert_eq!(node, Node::Link(Link::Project("project".to_string())));
    }

    #[test]
    fn test_project_empty_page_is_project_link() {
        // [/project/] has empty page — treated as project link
        let node = parse("[/project/]");
        assert_eq!(node, Node::Link(Link::Project("project".to_string())));
    }
}
