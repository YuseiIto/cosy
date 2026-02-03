use crate::{ExtensionParser, Node};
use winnow::combinator::{alt, repeat};
use winnow::prelude::*;
use winnow::Result as PResult;

mod bracket;
mod text;

use bracket::parse_bracket;
use text::parse_text;

pub fn parse<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Vec<Node<E::Output>>>
where
    E: ExtensionParser,
{
    repeat(
        0..,
        alt((
            // Parse bracketed content
            parse_bracket(extension),
            // Parse plain text
            parse_text,
        )),
    )
    .parse_next(input)
}
