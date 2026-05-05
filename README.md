# cosy

[![Crates.io](https://img.shields.io/crates/v/cosy.svg)](https://crates.io/crates/cosy)
[![Documentation](https://docs.rs/cosy/badge.svg)](https://docs.rs/cosy)
[![CI](https://github.com/YuseiIto/cosy/actions/workflows/ci.yml/badge.svg)](https://github.com/YuseiIto/cosy/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A parser for [Cosense](https://cosen.se/) (formerly Scrapbox) markup syntax that produces a typed AST.

## Installation

```toml
[dependencies]
cosy = "0.1"
```

To enable JSON serialization of the AST via [serde](https://serde.rs/):

```toml
[dependencies]
cosy = { version = "0.1", features = ["serde"] }
```

## Usage

```rust
use cosy::ast::{BlockContent, Node};

let doc = cosy::parse("Hello [* world]! See [https://example.com].", &()).unwrap();

for block in &doc {
    if let BlockContent::Line(nodes) = &block.content {
        for node in nodes {
            println!("{node:?}");
        }
    }
}
```

### Custom syntax extensions

Implement [`CosyParserExtension`](https://docs.rs/cosy/latest/cosy/trait.CosyParserExtension.html) to inject your own bracket syntax:

```rust
use cosy::CosyParserExtension;

#[derive(Debug, PartialEq)]
enum MySyntax { Highlight(String) }

struct MyExt;
impl CosyParserExtension for MyExt {
    type Output = MySyntax;
    fn parse_bracket(&self, content: &str) -> Option<MySyntax> {
        content.strip_prefix("! ").map(|s| MySyntax::Highlight(s.to_string()))
    }
}

let doc = cosy::parse("[! important]", &MyExt).unwrap();
```

See [`examples/speech_bubble_extension.rs`](examples/speech_bubble_extension.rs) for a full example.

### URL handling

Labeled links (`[label https://...]`) carry a parsed [`url::Url`](https://docs.rs/url) in
`Link::WithLabel.href`. `cosy` re-exports the `Url` type at its crate root for convenience:

```rust
use cosy::Url;
```

## Supported syntax

### Block-level

| Syntax | Description |
|--------|-------------|
| (indented lines) | Nested bullet list |
| `code:filename` | Code block |
| `table:name` | Table |
| `> text` | Quote |
| `? query` | Helpfeel search query |
| `$ command` `% command` | Command-line notation (bash / csh) |

### Inline

| Syntax | Description |
|--------|-------------|
| `[Page Name]` | Internal page link |
| `[/project/page]` | Cross-project link |
| `[https://...]` | External URL |
| `[URL Label]` | Labeled link |
| `[image.png]` | Image (detected by MIME type) |
| `[img link]` | Linked image |
| `[name.icon]` `[name.icon*3]` | User icon |
| `` `code` `` | Inline code |
| `[$ expr]` | Math (LaTeX) |
| `[* bold]` `[/ italic]` `[- strike]` | Decoration |
| `[[text]]` | Strong (large bold) text |
| `[[image.png]]` | Strong (large) image |
| `[[https://...]]` | Strong (large) link |
| `[[name.icon]]` | Strong (large) icon |
| `#tag` | Hashtag |
| `[N35.xx,E139.xx]` `[N35.xx,E139.xx,Z14]` | Geographic coordinate (map embed) |


## License

MIT — see [LICENSE](LICENSE).
