use crate::ExtensionParser;
use crate::ast::Link;
use crate::ast::Node;
use crate::tokens::{LBRACKET, RBRACKET};
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::take_until;

use super::node::parse_nodes;

pub fn parse_bracket<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    E: ExtensionParser,
{
    move |input: &mut &'i str| {
        let content: &str =
            delimited(LBRACKET, take_until(0.., RBRACKET), RBRACKET).parse_next(input)?;

        if let Some(custom_node) = extension.parse_bracket(content) {
            return Ok(Node::Custom(custom_node));
        }

        // Handle specific bracket types that need recursion or context

        // 1. Decoration: [* bold], [*- bold strike]
        // Condition: Starts with decoration chars followed by space
        // We define decoration chars as sequence of *, -, /, _, !
        // Simple check: take while matches decoration char
        let mut chars_iter = content.chars();
        let first_char = chars_iter.next();

        if let Some(c) = first_char {
            if c == '$' {
                // Math
                let math_content = &content[1..];
                return Ok(Node::Math(math_content.trim().to_string()));
            } else if is_decoration_char(c) {
                // Check if it's a decoration pattern: "decos "
                // We need to find the first space
                if let Some((decos, body)) = content.split_once(' ') {
                    if decos.chars().all(is_decoration_char) {
                        // It is a decoration
                        let mut body_input = body;
                        let nodes = parse_nodes(&mut body_input, extension)?;
                        return Ok(Node::Decoration {
                            decos: decos.to_string(),
                            nodes,
                        });
                    }
                }
            }
        }

        // 2. Icon: [name.icon] or [name.icon*3]
        if content.ends_with(".icon") {
            // Simple icon
            let name = content.trim_end_matches(".icon");
            return Ok(Node::Icon {
                name: name.to_string(),
                count: 1,
            });
        }
        // TODO: Handle repetition [name.icon*3] if needed.

        // 3. Links (recurse on label)
        // Split by space
        if let Some((left, right)) = content.split_once(' ') {
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
        if is_image_url(content) {
            Ok(Node::Image(content.to_string()))
        } else if is_url(content) {
            Ok(Node::Link(Link::Url(content.to_string())))
        } else {
            Ok(Node::Link(Link::Page(content.to_string())))
        }
    }
}

fn is_decoration_char(c: char) -> bool {
    matches!(c, '*' | '-' | '/' | '_' | '!')
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn is_image_url(s: &str) -> bool {
    is_url(s)
        && (s.ends_with(".png")
            || s.ends_with(".jpg")
            || s.ends_with(".gif")
            || s.ends_with(".webp"))
}
