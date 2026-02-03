use super::node::parse_nodes;
use crate::ExtensionParser;
use crate::ast::{Block, BlockContent};
use winnow::Result as PResult;
use winnow::prelude::*;
use winnow::token::{any, take_till};

pub fn parse_quote<'s, E>(
    input: &mut &'s str,
    extension: &'s E,
    indent: usize,
) -> PResult<Block<E::Output>>
where
    E: ExtensionParser,
{
    // Consume '>'
    let _ = any.parse_next(input)?;

    // Determine content (rest of line)
    let line_content = take_till(0.., |c| c == '\n').parse_next(input)?;

    // Consume newline if present
    if !input.is_empty() && (*input).starts_with('\n') {
        let _ = any.parse_next(input)?;
    }

    // Parse nodes in the content (trim start space usually after >?)
    let mut span = line_content;
    // Optional: trim leading space if exists? `> text` vs `>text`
    if span.starts_with(' ') {
        span = &span[1..];
    }

    let nodes = parse_nodes(&mut span, extension)?;

    Ok(Block {
        indent,
        content: BlockContent::Quote(nodes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn parse_quote_block() {
        let input = "> This is a quote line.\n";
        let mut input_stream = input;
        let result = parse_quote(&mut input_stream, &(), 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::Quote(vec![Node::Text("This is a quote line.".to_string())])
        );
    }

    #[test]
    fn parse_formatted_quote() {
        let input = "> [* Bold Quote] and [Linked part]\n";
        let mut input_stream = input;
        let result = parse_quote(&mut input_stream, &(), 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::Quote(vec![
                Node::Decoration {
                    decos: "*".to_string(),
                    nodes: vec![Node::Text("Bold Quote".to_string())]
                },
                Node::Text(" and ".to_string()),
                Node::Link(Link::Page("Linked part".to_string())),
            ])
        );
    }
}
