use crate::CosyParserExtension;
use crate::ast::{Block, BlockContent, CodeBlockMeta};
use crate::tokens::CODE_PREFIX;
use winnow::Result as PResult;
use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::opt;
use winnow::prelude::*;

pub fn parse_code_block<E>(input: &mut &str, indent: usize) -> PResult<Block<E::Output>>
where
    E: CosyParserExtension,
{
    // "code:filename"
    let _ = { CODE_PREFIX }.parse_next(input)?;
    let filename_line = till_line_ending.parse_next(input)?;
    let filename_line = if filename_line.trim().is_empty() {
        None
    } else {
        Some(filename_line.trim().to_string())
    };

    let meta = match filename_line {
        Some(ref line) => {
            // Check for (filetype) at end
            if let Some(start_idx) = line.rfind('(')
                && line.ends_with(')')
                && start_idx < line.len() - 1
            {
                let fname = line[..start_idx].trim().to_string();
                let ftype = line[start_idx + 1..line.len() - 1].trim().to_string();

                match (fname, ftype) {
                    (f, t) if !f.is_empty() && !t.is_empty() => CodeBlockMeta::Both {
                        filename: f,
                        language: t,
                    },
                    (f, _) if !f.is_empty() => CodeBlockMeta::NameOrLang(f),
                    (_, t) if !t.is_empty() => CodeBlockMeta::NameOrLang(t),
                    _ => CodeBlockMeta::None,
                }
            } else {
                CodeBlockMeta::NameOrLang(line.clone())
            }
        }
        None => CodeBlockMeta::None,
    };

    let _ = opt(line_ending).parse_next(input)?;

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
        let line = till_line_ending.parse_next(input)?;

        content.push_str(line);
        content.push('\n');

        if opt(line_ending).parse_next(input)?.is_none() {
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
        content: BlockContent::CodeBlock { meta, content },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn parse_code_block_with_filename() {
        let input = r#"code:example.rs
 fn main() {
  println!("Hello, world!");
 }
"#;

        let mut input_stream = input;
        let result = parse_code_block::<()>(&mut input_stream, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::CodeBlock {
                meta: CodeBlockMeta::NameOrLang("example.rs".to_string()),
                content: "fn main() {\nprintln!(\"Hello, world!\");\n}".to_string(),
            }
        );
    }

    #[test]
    fn parse_code_block_without_filename() {
        let input = "code:\n    print('Hello, World!')\n";
        let mut input_stream = input;
        let result = parse_code_block::<()>(&mut input_stream, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::CodeBlock {
                meta: CodeBlockMeta::None,
                content: "print('Hello, World!')".to_string(),
            }
        );
    }

    #[test]
    fn parse_code_block_with_filename_and_filetype() {
        let input = "code:script.py(python)\n def greet():\n print('Hello')\n";
        let mut input_stream = input;
        let result = parse_code_block::<()>(&mut input_stream, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::CodeBlock {
                meta: CodeBlockMeta::Both {
                    filename: "script.py".to_string(),
                    language: "python".to_string(),
                },
                content: "def greet():\nprint('Hello')".to_string(),
            }
        );
    }

    #[test]
    fn test_end_of_code_block() {
        let input = "code:example.py\n    print('Hello, World!')\nThis is outside the code block.";
        let mut input_stream = input;
        let result = parse_code_block::<()>(&mut input_stream, 0);
        assert!(result.is_ok());
        let block = result.unwrap();
        assert_eq!(block.indent, 0);
        assert_eq!(
            block.content,
            BlockContent::CodeBlock {
                meta: CodeBlockMeta::NameOrLang("example.py".to_string()),
                content: "print('Hello, World!')".to_string(),
            }
        );
    }
}
