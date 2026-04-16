use crate::ast::{Block, BlockContent};
use winnow::Result as PResult;
use winnow::prelude::*;
use winnow::token::{any, take_till};

pub fn parse_commandline<T>(input: &mut &str, indent: usize) -> PResult<Block<T>> {
    // Consume '$'
    let _ = any.parse_next(input)?;
    // Consume mandatory space
    let _ = any.parse_next(input)?;

    let content = take_till(0.., |c| c == '\n').parse_next(input)?;

    // Consume trailing newline if present
    if !input.is_empty() && (*input).starts_with('\n') {
        let _ = any.parse_next(input)?;
    }

    Ok(Block {
        indent,
        content: BlockContent::CommandLine(content.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn parse_commandline_basic_with_newline() {
        let mut input = "$ cargo build\n";
        let result = parse_commandline::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::CommandLine("cargo build".to_string())
        );
    }

    #[test]
    fn parse_commandline_no_newline() {
        let mut input = "$ ls -la";
        let result = parse_commandline::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(
            block.content,
            BlockContent::CommandLine("ls -la".to_string())
        );
    }

    #[test]
    fn parse_commandline_empty_command() {
        let mut input = "$ \n";
        let result = parse_commandline::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.content, BlockContent::CommandLine("".to_string()));
    }

    #[test]
    fn parse_commandline_indented() {
        let mut input = "$ echo hello\n";
        let result = parse_commandline::<()>(&mut input, 2);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 2);
        assert_eq!(
            block.content,
            BlockContent::CommandLine("echo hello".to_string())
        );
    }

    #[test]
    fn parse_commandline_special_chars_not_parsed() {
        let mut input = "$ git commit -m \"[fix] bug\"\n";
        let result = parse_commandline::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(
            block.content,
            BlockContent::CommandLine("git commit -m \"[fix] bug\"".to_string())
        );
    }
}
