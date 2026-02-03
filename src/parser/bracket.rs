use crate::tokens::RBRACKET;
use crate::{ExtensionParser, Node};
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::take_until;

pub fn parse_bracket<'s, E>(
    extension: &'s E,
) -> impl Parser<&'s str, Node<E::Output>, ContextError> + 's
where
    E: ExtensionParser,
{
    // delimited(開始, 中身, 終了)
    delimited(
        '[',
        take_until(0.., RBRACKET), // "]" が来るまで読む
        ']',
    )
    .map(move |content: &str| {
        // 1. Try if the content matches the custom extension
        if let Some(custom) = extension.parse_bracket(content) {
            Node::Custom(custom)
        } else {
            // 2. Otherwise, treat it as a simple link
            Node::Link(content.to_string())
        }
    })
}
