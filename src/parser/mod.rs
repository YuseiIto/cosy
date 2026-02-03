use crate::ast::Document;
use crate::ExtensionParser;
use winnow::combinator::repeat;
use winnow::prelude::*;
use winnow::Result as PResult;

mod block;
mod bracket;
mod node;
mod text;

use block::parse_block;

pub fn parse<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Document<E::Output>>
where
    E: ExtensionParser,
{
    repeat(0.., |i: &mut &'s str| parse_block(i, extension)).parse_next(input)
}
