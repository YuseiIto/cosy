use crate::CosyParserExtension;
use crate::ast::Link;
use crate::ast::Node;
use crate::tokens::{DECO_CHARS, DOLLAR, ICON_SUFFIX, LBRACKET, RBRACKET};
use crate::url::{UrlKind, infer_url_kind, is_url};
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::take_until;

use super::node::parse_nodes;

pub fn parse_bracket<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    E: CosyParserExtension,
{
    move |input: &mut &'i str| {
        let content: &str =
            delimited(LBRACKET, take_until(0.., RBRACKET), RBRACKET).parse_next(input)?;

        // 2. Icon: [name.icon] or [name.icon*3]
        if content.ends_with(ICON_SUFFIX) {
            // Simple icon
            let name = content.trim_end_matches(ICON_SUFFIX);
            return Ok(Node::Icon {
                name: name.to_string(),
                count: 1,
            });
        }
        // TODO: Handle repetition [name.icon*3] if needed.

        // 3. Links (recurse on label)
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
