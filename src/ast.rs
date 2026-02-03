#[derive(Debug, PartialEq, Clone)]
pub enum Node<T> {
    Text(String),
    Link(String), // 通常のリンク [page]
    Custom(T),    // 拡張記法
}
