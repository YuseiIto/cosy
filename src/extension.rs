/// A trait that enables parsing of user-defined syntax extensions.
///
/// This trait allows users to inject custom syntax handling into the parser.
/// It provides hooks for parsing content within brackets (`[...]`) and for
/// defining custom block-level elements.
/// Hooks are called before the default parsers.
///
/// Implementors can define their own output type, which will be wrapped in
/// the AST nodes.
pub trait CosyParserExtension {
    /// The type of the output produced by the custom parser.
    /// This type will be stored in [`crate::ast::Node::Custom`] or [`crate::ast::BlockContent::Custom`].
    type Output;

    /// Parses the content inside brackets and returns an optional custom output.
    ///
    /// This method is called when the parser encounters a bracketed sequence.
    /// If this method returns `Some`, the content is treated as a custom node.
    /// If it returns `None`, the parser attempts to parse it as standard syntax
    /// (e.g., links, decorations).
    ///
    /// # Arguments
    ///
    /// * `content` - The string content inside the brackets (excluding the brackets themselves).
    ///
    /// # Returns
    ///
    /// * `Option<Self::Output>` - The custom parsed object if successful, or `None`.
    fn parse_bracket(&self, content: &str) -> Option<Self::Output>;

    /// Parses block-level content and returns an optional custom output.
    ///
    /// This method allows for defining entire blocks that follow custom rules.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the block.
    ///
    /// # Returns
    ///
    /// * `Option<Self::Output>` - The custom parsed object if successful, or `None`.
    fn parse_block(&self, content: &str) -> Option<Self::Output>;
}

impl CosyParserExtension for () {
    type Output = ();
    fn parse_bracket(&self, _content: &str) -> Option<Self::Output> {
        None
    }
    fn parse_block(&self, _content: &str) -> Option<Self::Output> {
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

        fn parse_block(&self, _content: &str) -> Option<Self::Output> {
            None
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
