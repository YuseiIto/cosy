use crate::ast::Link;
use crate::ast::Node;
use crate::tokens::{LBRACKET, RBRACKET};
use crate::ExtensionParser;
use winnow::combinator::delimited;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::token::take_until;

pub fn parse_bracket<'s, 'i, E>(
    extension: &'s E,
) -> impl Parser<&'i str, Node<E::Output>, ContextError> + 's
where
    E: ExtensionParser,
{
    move |input: &mut &'i str| {
        let content: &str =
            delimited(LBRACKET, take_until(0.., RBRACKET), RBRACKET).parse_next(input)?;

        if let Some(custom_node) = extension.parse_bracket(content) {
            return Ok(Node::Custom(custom_node));
        }

        // 2. 中身を解析してノードに分類する
        Ok(analyze_bracket_content(content))
    }
}

fn analyze_bracket_content<T>(content: &str) -> Node<T> {
    // 空ならただのテキスト扱い（あるいは空ノード）
    if content.is_empty() {
        return Node::Text("".to_string());
    }

    // A. 拡張機能などのチェック (Custom)
    // ここで T のパーサーを呼ぶフックを入れる（今回は省略）

    // B. スペースで区切られている場合 ( [link label] or [label link] or [page name] )
    if let Some((left, right)) = content.split_once(' ') {
        let left = left.trim();
        let right = right.trim(); // split_onceは区切り文字を消費するが、前後の空白除去は必要

        if is_url(left) {
            // https://www.merriam-webster.com/dictionary/label -> URLが左
            return create_link_with_label(left, right);
        } else if is_url(right) {
            // [Label URL] -> URLが右
            return create_link_with_label(right, left);
        } else {
            // 両方ともURLじゃない -> ただのスペース入りのページ名
            // 例: [My Project Page]
            return Node::Link(Link::Page(content.to_string()));
        }
    }

    // C. スペースがない場合
    if is_image_url(content) {
        // 画像URLそのもの -> 画像ノード
        Node::Image(content.to_string())
    } else if is_url(content) {
        // 普通のURL -> 外部リンク
        Node::Link(Link::Url(content.to_string()))
    } else {
        // それ以外 -> 内部リンク (Page)
        Node::Link(Link::Page(content.to_string()))
    }
}

// ラベル付きリンクを生成するヘルパー
// ラベル部分はさらに Node として解析したい場合（画像を含むラベルなど）、
// ここで再帰的にパースを呼ぶことも可能だが、今回はシンプルにTextにする
fn create_link_with_label<T>(url: &str, label: &str) -> Node<T> {
    // もしラベル自体が画像URLなら「画像リンク」になるが、
    // ここでは単純化のため「ラベル付きリンク」として定義
    // 必要ならここで is_image_url(label) をチェックして構造を変える

    Node::Link(Link::WithLabel {
        href: url.to_string(),
        // AST定義に合わせて Vec<Node> に包む
        label: vec![Node::Text(label.to_string())],
    })
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn is_image_url(s: &str) -> bool {
    // 簡易的な判定。実際は拡張子チェックなどを厳密にやる
    is_url(s)
        && (s.ends_with(".png")
            || s.ends_with(".jpg")
            || s.ends_with(".gif")
            || s.ends_with(".webp"))
}
