use super::{code, line, quote, table};
use crate::CosyParserExtension;
use crate::ast::Block;
use crate::tokens::{CODE_PREFIX, GT, TABLE_PREFIX};
use winnow::Result as PResult;
use winnow::combinator::{eof, not};
use winnow::prelude::*;

pub fn parse_block<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Block<E::Output>>
where
    E: CosyParserExtension,
{
    // Ensure not EOF
    let _ = not(eof).parse_next(input)?;

    // 1. Calculate and consume indent
    let indent_len = input.chars().take_while(|&c| c == ' ').count();
    if indent_len > 0 {
        let _ = winnow::token::take(indent_len).parse_next(input)?;
    }

    // 2. Determine block type
    // We look at the immediate content
    if (*input).starts_with(CODE_PREFIX) {
        return code::parse_code_block::<E>(input, indent_len);
    }
    if (*input).starts_with(TABLE_PREFIX) {
        return table::parse_table(input, extension, indent_len);
    }
    if (*input).starts_with(GT) {
        return quote::parse_quote(input, extension, indent_len);
    }

    // Default: Line
    line::parse_line(input, extension, indent_len)
}
