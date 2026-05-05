use super::{code, commandline, helpfeel, line, quote, table};
use crate::CosyParserExtension;
use crate::ast::Block;
use crate::tokens::{
    CMD_PREFIX_DOLLAR, CMD_PREFIX_PERCENT, CODE_PREFIX, GT, HELPFEEL_PREFIX, TABLE_PREFIX,
};
use winnow::Result as PResult;
use winnow::combinator::{eof, not};
use winnow::prelude::*;

pub fn parse_block<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Block<E::Output>>
where
    E: CosyParserExtension,
{
    // Ensure not EOF
    not(eof).parse_next(input)?;

    // 1. Calculate and consume indent
    let indent_len = input.chars().take_while(|&c| c == ' ').count();
    if indent_len > 0 {
        let _ = winnow::token::take(indent_len).parse_next(input)?;
    }

    // 2. Determine block type and dispatch
    if input.starts_with(CODE_PREFIX) {
        code::parse_code_block::<E>(input, indent_len)
    } else if input.starts_with(TABLE_PREFIX) {
        table::parse_table(input, extension, indent_len)
    } else if input.starts_with(GT) {
        quote::parse_quote(input, extension, indent_len)
    } else if input.starts_with(HELPFEEL_PREFIX) {
        helpfeel::parse_helpfeel::<E::Output>(input, indent_len)
    } else if input.starts_with(CMD_PREFIX_DOLLAR) || input.starts_with(CMD_PREFIX_PERCENT) {
        commandline::parse_commandline::<E::Output>(input, indent_len)
    } else {
        line::parse_line(input, extension, indent_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::BlockContent;

    fn parse(input: &str) -> Block<()> {
        let mut s = input;
        parse_block(&mut s, &()).unwrap()
    }

    #[test]
    fn dispatch_code_block() {
        let block = parse("code:main.rs\n  fn main() {}\n");
        assert!(matches!(block.content, BlockContent::CodeBlock { .. }));
    }

    #[test]
    fn dispatch_table() {
        let block = parse("table:scores\n\tA\tB\n");
        assert!(matches!(block.content, BlockContent::Table { .. }));
    }

    #[test]
    fn dispatch_quote() {
        let block = parse(">hello\n");
        assert!(matches!(block.content, BlockContent::Quote(_)));
    }

    #[test]
    fn dispatch_helpfeel() {
        let block = parse("? search query\n");
        assert!(matches!(block.content, BlockContent::Helpfeel(_)));
    }

    #[test]
    fn dispatch_commandline_dollar() {
        let block = parse("$ cargo build\n");
        assert!(matches!(
            block.content,
            BlockContent::CommandLine {
                prompt: crate::ast::ShellPrompt::Dollar,
                ..
            }
        ));
    }

    #[test]
    fn dispatch_commandline_percent() {
        let block = parse("% ls -la\n");
        assert!(matches!(
            block.content,
            BlockContent::CommandLine {
                prompt: crate::ast::ShellPrompt::Percent,
                ..
            }
        ));
    }

    #[test]
    fn dispatch_line() {
        let block = parse("hello world\n");
        assert!(matches!(block.content, BlockContent::Line(_)));
    }

    #[test]
    fn indented_line() {
        let block = parse("  indented\n");
        assert_eq!(block.indent, 2);
        assert!(matches!(block.content, BlockContent::Line(_)));
    }

    #[test]
    fn eof_returns_error() {
        let mut s = "";
        assert!(parse_block(&mut s, &()).is_err());
    }
}
