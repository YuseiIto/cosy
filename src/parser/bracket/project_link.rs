use crate::ast::Link;
use crate::ast::Node;
use winnow::Result as PResult;
use winnow::combinator::{opt, preceded};
use winnow::prelude::*;
use winnow::token::take_till;

pub(super) fn parse_project_link<T>(input: &mut &str) -> PResult<Node<T>> {
    // [/project], [/project/], or [/project/page]
    preceded(
        '/',
        (take_till(1.., '/'), opt(preceded('/', winnow::token::rest))),
    )
    .map(|(project, page): (&str, Option<&str>)| {
        let project = project.to_string();
        match page {
            Some(p) if !p.is_empty() => Node::Link(Link::ProjectPage {
                project,
                page: p.to_string(),
            }),
            _ => Node::Link(Link::Project(project)),
        }
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
