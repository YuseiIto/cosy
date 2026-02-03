use cosy::ExtensionParser;

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

fn main() {
    let extension = MyExtension;
    let input = "こんにちは、[{ フキダシ] これは [テスト] です。";

    let mut input_stream = input;

    let result = cosy::parse(&mut input_stream, &extension);

    match result {
        Ok(nodes) => println!("{:#?}", nodes),
        Err(e) => println!("Error: {}", e),
    }
}
