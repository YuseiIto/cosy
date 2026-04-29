use crate::tokens::{LBRACKET, RBRACKET};
use winnow::error::ContextError;

/// Parses bracket content respecting nested `[`/`]` pairs.
/// Consumes everything up to the matching `]` at depth 0, without consuming the `]` itself.
pub fn take_bracket_content<'i>(input: &mut &'i str) -> Result<&'i str, ContextError> {
    let mut depth: usize = 0;
    let s = *input;
    for (i, c) in s.char_indices() {
        match c {
            LBRACKET => depth += 1,
            RBRACKET if depth == 0 => {
                let content = &s[..i];
                *input = &s[i..];
                return Ok(content);
            }
            RBRACKET => depth -= 1,
            _ => {}
        }
    }
    Err(ContextError::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(&str, &str), ()> {
        let mut s = input;
        let content = take_bracket_content(&mut s).map_err(|_| ())?;
        Ok((content, s))
    }

    #[test]
    fn simple_content() {
        let (content, rest) = parse("hello]").unwrap();
        assert_eq!(content, "hello");
        assert_eq!(rest, "]");
    }

    #[test]
    fn nested_brackets() {
        let (content, rest) = parse("outer [inner] end]").unwrap();
        assert_eq!(content, "outer [inner] end");
        assert_eq!(rest, "]");
    }

    #[test]
    fn deeply_nested() {
        let (content, rest) = parse("a [b [c] d] e]rest").unwrap();
        assert_eq!(content, "a [b [c] d] e");
        assert_eq!(rest, "]rest");
    }

    #[test]
    fn empty_content() {
        let (content, rest) = parse("]after").unwrap();
        assert_eq!(content, "");
        assert_eq!(rest, "]after");
    }

    #[test]
    fn unmatched_open_bracket_fails() {
        assert!(parse("hello [world").is_err());
    }
}
