use crate::ast::{Block, BlockContent};
use winnow::Result as PResult;
use winnow::prelude::*;
use winnow::token::{any, take_till};

pub fn parse_helpfeel<T>(input: &mut &str, indent: usize) -> PResult<Block<T>> {
    // Consume '?'
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
        content: BlockContent::Helpfeel(content.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn parse_helpfeel_basic_with_newline() {
        let mut input = "? how do I install rust\n";
        let result = parse_helpfeel::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::Helpfeel("how do I install rust".to_string())
        );
    }

    #[test]
    fn parse_helpfeel_no_newline() {
        let mut input = "? search query";
        let result = parse_helpfeel::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(
            block.content,
            BlockContent::Helpfeel("search query".to_string())
        );
    }

    #[test]
    fn parse_helpfeel_empty_query() {
        let mut input = "? \n";
        let result = parse_helpfeel::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.content, BlockContent::Helpfeel("".to_string()));
    }

    #[test]
    fn parse_helpfeel_special_chars_not_parsed() {
        let mut input = "? [search] #tag\n";
        let result = parse_helpfeel::<()>(&mut input, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(
            block.content,
            BlockContent::Helpfeel("[search] #tag".to_string())
        );
    }

    #[test]
    fn parse_helpfeel_indented() {
        let mut input = "? query\n";
        let result = parse_helpfeel::<()>(&mut input, 1);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 1);
        assert_eq!(block.content, BlockContent::Helpfeel("query".to_string()));
    }
}
