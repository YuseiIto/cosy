use crate::ast::Node;
use crate::tokens::{LBRACKET, RBRACKET, WHITESPACE};
use winnow::Result as PResult;
use winnow::prelude::*;
use winnow::token::take_while;

/// Parses `#タグ名` into `Node::Hashtag`.
/// Tag terminates at space, `[`, `]`, or end of input.
/// Fails if nothing follows `#` (falls back to parse_text).
pub fn parse_hashtag<'s, T>(input: &mut &'s str) -> PResult<Node<T>> {
    let _ = '#'.parse_next(input)?;
    let terminators = [WHITESPACE, LBRACKET, RBRACKET];
    let tag = take_while(1.., |c: char| !terminators.contains(&c)).parse_next(input)?;
    Ok(Node::Hashtag(tag.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hashtag_ascii() {
        let mut input = "#rust_lang rest";
        let result: Node<()> = parse_hashtag(&mut input).unwrap();
        assert_eq!(result, Node::Hashtag("rust_lang".to_string()));
        assert_eq!(input, " rest");
    }

    #[test]
    fn test_parse_hashtag_japanese() {
        let mut input = "#日本語タグ テキスト";
        let result: Node<()> = parse_hashtag(&mut input).unwrap();
        assert_eq!(result, Node::Hashtag("日本語タグ".to_string()));
        assert_eq!(input, " テキスト");
    }

    #[test]
    fn test_parse_hashtag_end_of_input() {
        let mut input = "#タグ";
        let result: Node<()> = parse_hashtag(&mut input).unwrap();
        assert_eq!(result, Node::Hashtag("タグ".to_string()));
        assert_eq!(input, "");
    }

    #[test]
    fn test_parse_hashtag_fails_on_lone_hash() {
        // # followed by space → parse_hashtag fails, falls through to parse_text
        let mut input = "# ";
        assert!(parse_hashtag::<()>(&mut input).is_err());
    }
}
