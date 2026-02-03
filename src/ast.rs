// ドキュメント全体
pub type Document<T> = Vec<Block<T>>;

// --------------------------------------------------------
// ブロックレベル (行単位の構造)
// --------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct Block<T> {
    pub indent: usize,
    pub content: BlockContent<T>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockContent<T> {
    Line(Vec<Node<T>>), // 通常の行（中身はインライン要素の列）

    CodeBlock {
        filename: Option<String>,
        indent: usize,
        content: String,
    },

    Table {
        name: String,
        rows: Vec<Vec<Node<T>>>, // セルの中身もインライン解析対象
    },

    Quote(Vec<Node<T>>), // 引用の中身もインライン解析対象

    // ★ ブロックレベルの拡張
    // 例: Youtube埋め込み、特殊なdivブロックなど
    Custom(T),
}

// --------------------------------------------------------
// インラインレベル (文字単位の構造)
// --------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub enum Node<T> {
    Text(String),

    // リンク（ラベルの中にさらにNodeが入るためTが必要）
    Link(Link<T>),

    Image(String),

    Icon {
        name: String,
        count: usize,
    },

    InlineCode(String),

    Math(String),

    // 装飾 (再帰構造なのでTが必要)
    Decoration {
        decos: String,       // "*", "*-" など
        nodes: Vec<Node<T>>, // 装飾の中身
    },

    // ★ インラインレベルの拡張
    // 例: [# 色付き], [! 警告バッジ], カスタム記法
    Custom(T),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Link<T> {
    Page(String),
    Url(String),
    WithLabel {
        href: String,
        label: Vec<Node<T>>, // ラベルの中にCustomが入る可能性もある！
    },
}
