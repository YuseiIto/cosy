// The trait for parsing custom extensions in a markup language
pub trait CosyParserExtension {
    type Output;
    // Parse the content inside brackets and return an optional custom output
    fn parse_bracket(&self, content: &str) -> Option<Self::Output>;
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
            if let Some(body) = content.strip_prefix("{ ") {
                Some(MySyntax::SpeechBubble(body.to_string()))
            } else {
                None
            }
        }

        fn parse_block(&self, _content: &str) -> Option<Self::Output> {
            None
        }
    }

    #[test]
    fn parse_speech_bubble() {
        let extension = MyExtension;
        let input = "こんにちは、[{ フキダシ] これは [テスト] です。";

        let mut input_stream = input;

        let result = crate::parse(&mut input_stream, &extension);

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
