use cosy::ast::*;

#[test]
fn parse_empty_input() {
    let doc = cosy::parse("", &()).unwrap();
    assert!(doc.is_empty());
}

#[test]
fn parse_single_line() {
    let doc = cosy::parse("Hello world\n", &()).unwrap();
    assert_eq!(doc.len(), 1);
    assert_eq!(doc[0].indent, 0);
    assert_eq!(
        doc[0].content,
        BlockContent::Line(vec![Node::Text("Hello world".to_string())])
    );
}

#[test]
fn parse_multiple_blocks() {
    let input = "First line\ncode:main.rs\n fn main() {}\n? search\n";
    let doc = cosy::parse(input, &()).unwrap();
    assert_eq!(doc.len(), 3);
    assert!(matches!(doc[0].content, BlockContent::Line(_)));
    assert!(matches!(doc[1].content, BlockContent::CodeBlock { .. }));
    assert!(matches!(doc[2].content, BlockContent::Helpfeel(_)));
}

#[test]
fn parse_nested_inline_syntax() {
    // Decoration containing a link
    let doc = cosy::parse("[* bold [link] text]\n", &()).unwrap();
    assert_eq!(doc.len(), 1);
    if let BlockContent::Line(nodes) = &doc[0].content {
        assert_eq!(nodes.len(), 1);
        if let Node::Decoration { decos, nodes } = &nodes[0] {
            assert_eq!(decos, "*");
            assert!(nodes.len() >= 3); // "bold ", Link, " text"
            assert!(
                nodes
                    .iter()
                    .any(|n| matches!(n, Node::Link(Link::Page(p)) if p == "link"))
            );
        } else {
            panic!("expected Decoration");
        }
    } else {
        panic!("expected Line");
    }
}

#[test]
fn parse_quote_with_inline() {
    let doc = cosy::parse(">Hello [link]\n", &()).unwrap();
    assert_eq!(doc.len(), 1);
    if let BlockContent::Quote(nodes) = &doc[0].content {
        assert!(nodes.len() >= 2);
        assert!(
            nodes
                .iter()
                .any(|n| matches!(n, Node::Link(Link::Page(p)) if p == "link"))
        );
    } else {
        panic!("expected Quote");
    }
}

#[test]
fn parse_indented_blocks() {
    let doc = cosy::parse(" item1\n  item2\n   item3\n", &()).unwrap();
    assert_eq!(doc.len(), 3);
    assert_eq!(doc[0].indent, 1);
    assert_eq!(doc[1].indent, 2);
    assert_eq!(doc[2].indent, 3);
}

#[test]
fn parse_strong_text() {
    let doc = cosy::parse("[[bold text]]\n", &()).unwrap();
    assert_eq!(doc.len(), 1);
    if let BlockContent::Line(nodes) = &doc[0].content {
        assert!(matches!(&nodes[0], Node::Strong(_)));
    } else {
        panic!("expected Line");
    }
}

#[test]
fn parse_coordinate() {
    let doc = cosy::parse("[N35.6578589,E139.7474797,Z14]\n", &()).unwrap();
    assert_eq!(doc.len(), 1);
    if let BlockContent::Line(nodes) = &doc[0].content {
        assert!(matches!(&nodes[0], Node::Coordinate { zoom: Some(14), .. }));
    } else {
        panic!("expected Line");
    }
}

#[cfg(feature = "serde")]
#[test]
fn serde_roundtrip() {
    let doc = cosy::parse("Hello [link] `code`\n", &()).unwrap();
    let json = serde_json::to_string(&doc).unwrap();
    let roundtrip: Document<()> = serde_json::from_str(&json).unwrap();
    assert_eq!(doc, roundtrip);
}
