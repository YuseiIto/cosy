use super::node::parse_nodes;
use crate::ExtensionParser;
use crate::ast::{Block, BlockContent};
use winnow::Result as PResult;
use winnow::prelude::*;
use winnow::token::{any, take_till};

pub fn parse_table<'s, E>(
    input: &mut &'s str,
    extension: &'s E,
    indent: usize,
) -> PResult<Block<E::Output>>
where
    E: ExtensionParser,
{
    // "table:name"
    let _ = "table:".parse_next(input)?;
    let name_line = take_till(0.., |c| c == '\n').parse_next(input)?;
    let name = name_line.trim().to_string();

    if !input.is_empty() && (*input).starts_with('\n') {
        let _ = any.parse_next(input)?;
    }

    let mut rows = Vec::new();

    loop {
        // Peek next line indent
        let current_indent = input.chars().take_while(|&c| c == ' ').count();
        if current_indent <= indent {
            break;
        }

        // Consume indent
        let _ = winnow::token::take(current_indent).parse_next(input)?;

        // Consume line
        let line = take_till(0.., |c| c == '\n').parse_next(input)?;

        // Parse row cells (tab separated)
        let cells_str: Vec<&str> = line.split('\t').collect();
        let mut cells = Vec::new();
        for mut cell_str in cells_str {
            // trim? usually tables align. Let's just parse.
            let nodes = parse_nodes(&mut cell_str, extension)?;
            cells.push(nodes);
        }
        rows.push(cells);

        if !input.is_empty() && (*input).starts_with('\n') {
            let _ = any.parse_next(input)?;
        } else {
            break;
        }
    }

    Ok(Block {
        indent,
        content: BlockContent::Table { name, rows },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn parse_table_block() {
        let input = "table:MyTable\n    Cell1\tCell2\tCell3\n    Data1\tData2\tData3\n";
        let mut input_stream = input;
        let result = parse_table(&mut input_stream, &(), 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::Table {
                name: "MyTable".to_string(),
                rows: vec![
                    vec![
                        vec![Node::Text("Cell1".to_string())],
                        vec![Node::Text("Cell2".to_string())],
                        vec![Node::Text("Cell3".to_string())],
                    ],
                    vec![
                        vec![Node::Text("Data1".to_string())],
                        vec![Node::Text("Data2".to_string())],
                        vec![Node::Text("Data3".to_string())],
                    ],
                ],
            }
        );
    }

    #[test]
    fn parse_table_without_name() {
        let input = "table:\n    A\tB\n    1\t2\n";
        let mut input_stream = input;
        let result = parse_table(&mut input_stream, &(), 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::Table {
                name: "".to_string(),
                rows: vec![
                    vec![
                        vec![Node::Text("A".to_string())],
                        vec![Node::Text("B".to_string())],
                    ],
                    vec![
                        vec![Node::Text("1".to_string())],
                        vec![Node::Text("2".to_string())],
                    ],
                ],
            }
        );
    }
}
