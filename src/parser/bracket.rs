use super::bracket_content::take_bracket_content;
use crate::CosyParserExtension;
use crate::ast::Node;
use crate::tokens::{LBRACKET, RBRACKET};
use winnow::combinator::{alt, delimited};
use winnow::error::ContextError;
use winnow::prelude::*;

mod coordinate;
mod icon;
mod links_and_pages;
mod math;
mod project_link;

use coordinate::parse_coordinate;
use icon::parse_icon;
use links_and_pages::parse_links_and_pages;
use math::parse_math;
use project_link::parse_project_link;

pub fn parse_bracket<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    'i: 's,
    E: CosyParserExtension,
{
    delimited(LBRACKET, take_bracket_content, RBRACKET).and_then(alt((
        parse_math,
        parse_icon,
        parse_project_link,
        parse_coordinate,
        parse_links_and_pages(extension),
    )))
}
