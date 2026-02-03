//! Inline-level AST nodes.

// --------------------------------------------------------
// Inline level (character-based structure)
// --------------------------------------------------------

/// Represents an inline-level element (node) within a block.
#[derive(Debug, PartialEq, Clone)]
pub enum Node<T> {
    /// Plain text.
    Text(String),

    /// A link (internal or external).
    ///
    /// Requires `T` because the label contains `Node`s.
    Link(Link<T>),

    /// An image reference.
    Image(String),

    /// An icon reference.
    Icon {
        /// The name of the icon.
        name: String,
        /// The repetition count of the icon.
        count: usize,
    },

    /// Inline code snippet.
    InlineCode(String),

    /// Mathematical expression.
    Math(String),

    /// Decorated text (bold, italic, etc.).
    ///
    /// Requires `T` due to recursive structure.
    Decoration {
        /// The decoration characters (e.g., "*", "*-").
        decos: String,
        /// The content inside the decoration.
        nodes: Vec<Node<T>>,
    },

    /// A custom inline-level extension.
    ///
    /// This allows for extending the parser with custom inline types (e.g., colored text, warning badges).
    Custom(T),
}

/// Represents a link target and optional label.
#[derive(Debug, PartialEq, Clone)]
pub enum Link<T> {
    /// A link to another page (internal link).
    Page(String),
    /// A raw URL (external link).
    Url(String),
    /// A link with an explicit label.
    WithLabel {
        /// The destination URL or page name.
        href: String,
        /// The label content, which can contain other inline nodes.
        ///
        /// The label might contain `Custom` nodes.
        label: Vec<Node<T>>,
    },
}
