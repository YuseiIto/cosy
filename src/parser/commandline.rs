use crate::ast::{Block, BlockContent, ShellPrompt};
use winnow::Result as PResult;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::opt;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::any;

pub fn parse_commandline<T>(input: &mut &str, indent: usize) -> PResult<Block<T>> {
    // Determine prompt from the first character.
    let prompt = match input.chars().next() {
        Some('$') => ShellPrompt::Dollar,
        Some('%') => ShellPrompt::Percent,
        _ => return Err(ContextError::new()),
    };

    // Consume the prompt character
    let _ = any.parse_next(input)?;
    // Consume mandatory space
    let _ = any.parse_next(input)?;

    let content = till_line_ending.parse_next(input)?;
    let _ = opt(line_ending).parse_next(input)?;

    Ok(Block {
        indent,
        content: BlockContent::CommandLine {
            prompt,
            command: content.to_string(),
        },
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
            BlockContent::CommandLine {
                prompt: ShellPrompt::Dollar,
                command: "cargo build".to_string(),
            }
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
            BlockContent::CommandLine {
                prompt: ShellPrompt::Dollar,
                command: "ls -la".to_string(),
            }
        );
    }

    #[test]
    fn parse_commandline_empty_command() {
        let mut input = "$ \n";
        let result = parse_commandline::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(
            block.content,
            BlockContent::CommandLine {
                prompt: ShellPrompt::Dollar,
                command: "".to_string(),
            }
        );
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
            BlockContent::CommandLine {
                prompt: ShellPrompt::Dollar,
                command: "echo hello".to_string(),
            }
        );
    }

    #[test]
    fn parse_csh_commandline_basic() {
        let mut input = "% ls -la\n";
        let result = parse_commandline::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(
            block.content,
            BlockContent::CommandLine {
                prompt: ShellPrompt::Percent,
                command: "ls -la".to_string(),
            }
        );
    }

    #[test]
    fn parse_csh_commandline_indented() {
        let mut input = "% echo hello\n";
        let result = parse_commandline::<()>(&mut input, 1);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 1);
        assert_eq!(
            block.content,
            BlockContent::CommandLine {
                prompt: ShellPrompt::Percent,
                command: "echo hello".to_string(),
            }
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
            BlockContent::CommandLine {
                prompt: ShellPrompt::Dollar,
                command: "git commit -m \"[fix] bug\"".to_string(),
            }
        );
    }
}
