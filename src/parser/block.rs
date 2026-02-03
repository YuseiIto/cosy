use super::line;
use crate::ast::{Block, BlockContent};
use crate::ExtensionParser;
use winnow::combinator::{eof, not};
use winnow::prelude::*;
use winnow::token::{any, take_till};
use winnow::Result as PResult;

use super::node::parse_nodes;

pub fn parse_block<'s, E>(input: &mut &'s str, extension: &'s E) -> PResult<Block<E::Output>>
where
    E: ExtensionParser,
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
    if (*input).starts_with("code:") {
        return parse_code_block::<E>(input, indent_len);
    }
    if (*input).starts_with("table:") {
        return parse_table(input, extension, indent_len);
    }
    if (*input).starts_with('>') {
        return parse_quote(input, extension, indent_len);
    }

    // Default: Line
    line::parse_line(input, extension, indent_len)
}

fn parse_quote<'s, E>(
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

fn parse_code_block<'s, E>(input: &mut &'s str, indent: usize) -> PResult<Block<E::Output>>
where
    E: ExtensionParser,
{
    // "code:filename"
    let _ = "code:".parse_next(input)?;
    let filename_line = take_till(0.., |c| c == '\n').parse_next(input)?;
    let filename = if filename_line.trim().is_empty() {
        None
    } else {
        Some(filename_line.trim().to_string())
    };

    if !input.is_empty() && (*input).starts_with('\n') {
        let _ = any.parse_next(input)?;
    }

    // Parse subsequent lines that are indented MORE than `indent`
    // We assume the block continues as long as lines are indented > indent.
    let mut content = String::new();

    loop {
        // Peek next line indent
        let current_indent = input.chars().take_while(|&c| c == ' ').count();

        if current_indent <= indent {
            // Block ended
            break;
        }

        // It is part of the block
        // Consume indent
        let _ = winnow::token::take(current_indent).parse_next(input)?;

        // Consume line
        let line = take_till(0.., |c| c == '\n').parse_next(input)?;

        content.push_str(line);
        content.push('\n');

        if !input.is_empty() && (*input).starts_with('\n') {
            let _ = any.parse_next(input)?;
        } else {
            // End of input
            break;
        }
    }

    // Remove last newline if added
    if content.ends_with('\n') {
        content.pop();
    }

    Ok(Block {
        indent,
        content: BlockContent::CodeBlock {
            filename,
            indent: indent + 1, // Assumed content indent
            content,
        },
    })
}

fn parse_table<'s, E>(
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
