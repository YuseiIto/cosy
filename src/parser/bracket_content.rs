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
