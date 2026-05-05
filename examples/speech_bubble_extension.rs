//! An example of extending the `cosy` parser with custom syntax.
//!
//! This example demonstrates how to implement the `CosyParserExtension` trait
//! to support a custom "speech bubble" notation (e.g., `[{ content]`).

use cosy::CosyParserExtension;

/// Represents custom syntax elements supported by this extension.
#[derive(Debug, PartialEq)]
enum MySyntax {
    /// A speech bubble containing text.
    SpeechBubble(String),
}

/// A custom extension for the `cosy` parser.
struct MyExtension;

impl CosyParserExtension for MyExtension {
    type Output = MySyntax;

    /// Parses custom bracketed content.
    ///
    /// If the content starts with `{ `, it is interpreted as a `SpeechBubble`.
    /// Returning `None` falls back to the default parser.
    fn parse_bracket(&self, content: &str) -> Option<Self::Output> {
        content
            .strip_prefix("{ ")
            .map(|body| MySyntax::SpeechBubble(body.to_string()))
    }
}

fn main() {
    let extension = MyExtension;
    let input = "Cheshire Cat[{ We're all mad here.]";

    let result = cosy::parse(input, &extension);

    match result {
        Ok(nodes) => println!("{:#?}", nodes),
        Err(e) => println!("Error: {}", e),
    }
}
