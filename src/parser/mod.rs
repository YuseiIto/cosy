use crate::CosyParserExtension;
use crate::ast::Document;
use winnow::Result as PResult;
use winnow::combinator::repeat;
use winnow::prelude::*;

mod block;
mod bracket;
mod code;
mod code_inline;
mod line;
mod node;
mod quote;
mod table;
mod text;

use block::parse_block;

pub fn parse<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Document<E::Output>>
where
    E: CosyParserExtension,
{
    repeat(0.., |i: &mut &'s str| parse_block(i, extension)).parse_next(input)
}
