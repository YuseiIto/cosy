// The trait for parsing custom extensions in a markup language
pub trait ExtensionParser {
    type Output;
    // Parse the content inside brackets and return an optional custom output
    fn parse_bracket(&self, content: &str) -> Option<Self::Output>;
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
    impl ExtensionParser for MyExtension {
        type Output = MySyntax;
        fn parse_bracket(&self, content: &str) -> Option<Self::Output> {
            if let Some(body) = content.strip_prefix("{ ") {
                Some(MySyntax::SpeechBubble(body.to_string()))
            } else {
                None
            }
        }
    }

    #[test]
    fn parse_speech_bubble() {
        let extension = MyExtension;
        let input = "こんにちは、[{ フキダシ] これは [テスト] です。";

        let mut input_stream = input;

        let result = crate::parse(&mut input_stream, &extension);

        assert!(result.is_ok());
        let nodes = result.unwrap();
        assert_eq!(
            nodes,
            vec![
                Node::Text("こんにちは、".to_string()),
                Node::Custom(MySyntax::SpeechBubble("フキダシ".to_string())),
                Node::Text(" これは ".to_string()),
                Node::Link(Link::Page("テスト".to_string())),
                Node::Text(" です。".to_string()),
            ]
        );
    }
}
