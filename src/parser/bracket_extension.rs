use crate::CosyParserExtension;
use crate::ast::Link;
use crate::ast::Node;
use crate::tokens::{DECO_CHARS, DOLLAR, ICON_SUFFIX, LBRACKET, RBRACKET};
use crate::url::{UrlKind, infer_url_kind, is_url};
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{take_until, take_while};

use super::node::parse_nodes;

pub fn parse_bracket_extension<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    E: CosyParserExtension,
{
    move |input: &mut &'i str| {
        let content: &str =
            delimited(LBRACKET, take_until(0.., RBRACKET), RBRACKET).parse_next(input)?;
        match extension.parse_bracket(content) {
            None => Err(ContextError::new()),
            Some(custom_node) => Ok(Node::Custom(custom_node)),
        }
    }
}
