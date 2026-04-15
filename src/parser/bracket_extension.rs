use super::bracket_content::take_bracket_content;
use crate::CosyParserExtension;
use crate::ast::Node;
use crate::tokens::{LBRACKET, RBRACKET};
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;

pub fn parse_bracket_extension<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    E: CosyParserExtension,
{
    move |input: &mut &'i str| {
        let content: &str =
            delimited(LBRACKET, take_bracket_content, RBRACKET).parse_next(input)?;
        match extension.parse_bracket(content) {
            None => Err(ContextError::new()),
            Some(custom_node) => Ok(Node::Custom(custom_node)),
        }
    }
}
