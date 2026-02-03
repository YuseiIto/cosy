use crate::ExtensionParser;
use crate::ast::{Block, BlockContent};
use winnow::Result as PResult;
use winnow::prelude::*;
use winnow::token::{any, take_till};

use super::node::parse_nodes;

pub fn parse_line<'s, E>(
    input: &mut &'s str,
    extension: &'s E,
    indent: usize,
) -> PResult<Block<E::Output>>
where
    E: ExtensionParser,
{
    let line_content = take_till(0.., |c| c == '\n').parse_next(input)?;

    // Consume newline if present
    if !input.is_empty() && (*input).starts_with('\n') {
        let _ = any.parse_next(input)?;
    }

    let mut span = line_content;
    let nodes = parse_nodes(&mut span, extension)?;

    Ok(Block {
        indent,
        content: BlockContent::Line(nodes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn parse_root_line() {
        let input = "the simplest root string";
        let mut input_stream = input;
        let result = parse_line(&mut input_stream, &(), 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::Line(vec![Node::Text("the simplest root string".to_string())])
        );
    }

    #[test]
    fn parse_indented_line() {
        let input = "an indented line"; // Indentation is not consumed by parse_line
        let mut input_stream = input;
        let result = parse_line(&mut input_stream, &(), 4);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 4);
        assert_eq!(
            block.content,
            BlockContent::Line(vec![Node::Text("an indented line".to_string())])
        );
    }

    #[test]
    fn parse_with_node() {
        let input = "Hello, [world.icon]!";
        let mut input_stream = input;
        let result = parse_line(&mut input_stream, &(), 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::Line(vec![
                Node::Text("Hello, ".to_string()),
                Node::Icon {
                    name: "world".to_string(),
                    count: 1
                },
                Node::Text("!".to_string()),
            ])
        );
    }
}
