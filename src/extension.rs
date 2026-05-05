/// A trait that enables parsing of user-defined bracket syntax.
///
/// This trait allows users to inject custom syntax handling into the parser.
/// It provides a hook for parsing content within brackets (`[...]`) before
/// any built-in bracket parser is tried.
///
/// Implementors define their own [`Output`](Self::Output) type, which will be
/// stored in [`crate::ast::Node::Custom`] when [`parse_bracket`](Self::parse_bracket)
/// returns `Some`.
///
/// # Usage
///
/// Pass `&()` to [`crate::parse`] when no custom syntax is needed; the
/// blanket [`CosyParserExtension`] impl on `()` always returns `None`,
/// so all input falls through to the built-in parsers.
pub trait CosyParserExtension {
    /// The type of the output produced by the custom parser.
    ///
    /// This type is stored in [`crate::ast::Node::Custom`] when
    /// [`parse_bracket`](Self::parse_bracket) returns `Some`.
    type Output;

    /// Parses the content inside brackets and returns an optional custom output.
    ///
    /// This method is called **before** any built-in bracket parser. If it
    /// returns `Some`, the bracketed sequence becomes a [`crate::ast::Node::Custom`]
    /// holding the returned value, and built-in interpretation (links,
    /// decorations, icons, etc.) is skipped. If it returns `None`, the
    /// parser falls through to the built-in bracket parsers.
    ///
    /// # Arguments
    ///
    /// * `content` — the string between the brackets, with the surrounding
    ///   `[` and `]` removed but **otherwise unmodified** (no trimming, no
    ///   prefix stripping). Examples:
    ///
    ///   | Source       | `content`     |
    ///   |--------------|---------------|
    ///   | `[abc]`      | `"abc"`       |
    ///   | `[* bold]`   | `"* bold"`    |
    ///   | `[/ italic]` | `"/ italic"`  |
    ///   | `[$ x^2]`    | `"$ x^2"`     |
    ///
    /// # Returns
    ///
    /// * `Option<Self::Output>` — the custom parsed value if this
    ///   `content` is recognized by the extension, or `None` to fall back
    ///   to built-in parsers.
    ///
    /// # Note on recursion
    ///
    /// The returned `Self::Output` is treated as a terminal AST node —
    /// `cosy` does **not** recursively re-parse the contained text. If you
    /// need nested inline parsing, parse the relevant fragments yourself
    /// inside the extension before constructing `Self::Output`.
    fn parse_bracket(&self, content: &str) -> Option<Self::Output>;
}

impl CosyParserExtension for () {
    type Output = ();
    fn parse_bracket(&self, _content: &str) -> Option<Self::Output> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[derive(Debug, PartialEq)]
    enum MySyntax {
        SpeechBubble(String), // 吹き出し記法
    }

    struct MyExtension;
    impl CosyParserExtension for MyExtension {
        type Output = MySyntax;
        fn parse_bracket(&self, content: &str) -> Option<Self::Output> {
            content
                .strip_prefix("{ ")
                .map(|body| MySyntax::SpeechBubble(body.to_string()))
        }
    }

    #[test]
    fn parse_speech_bubble() {
        let extension = MyExtension;
        let input = "こんにちは、[{ フキダシ] これは [テスト] です。";

        let result = crate::parse(input, &extension);

        assert!(result.is_ok());
        let blocks = result.unwrap();

        assert_eq!(blocks.len(), 1);

        let block = &blocks[0];
        assert_eq!(block.indent, 0);

        let expected = BlockContent::Line(vec![
            Node::Text("こんにちは、".to_string()),
            Node::Custom(MySyntax::SpeechBubble("フキダシ".to_string())),
            Node::Text(" これは ".to_string()),
            Node::Link(Link::Page("テスト".to_string())),
            Node::Text(" です。".to_string()),
        ]);
        assert_eq!(block.content, expected);
    }
}
