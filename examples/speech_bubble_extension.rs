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
    fn parse_bracket(&self, content: &str) -> Option<Self::Output> {
        if let Some(body) = content.strip_prefix("{ ") {
            Some(MySyntax::SpeechBubble(body.to_string()))
        } else {
            None // Return none to fallback to the default parser
        }
    }

    /// Parses custom block content.
    ///
    /// This extension does not define any custom block-level syntax.
    fn parse_block(&self, _content: &str) -> Option<Self::Output> {
        None
    }
}

fn main() {
    let extension = MyExtension;
    let input = "Cheshire Cat[{ We're all mad here.]";

    let mut input_stream = input;

    let result = cosy::parse(&mut input_stream, &extension);

    match result {
        Ok(nodes) => println!("{:#?}", nodes),
        Err(e) => println!("Error: {}", e),
    }
}
