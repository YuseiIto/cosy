//! Block-level AST nodes.

use super::node::Node;

/// Represents a complete document, which is a sequence of blocks.
pub type Document<T> = Vec<Block<T>>;

// --------------------------------------------------------
// Block level (line-based structure)
// --------------------------------------------------------

/// Represents a block-level element in the document.
///
/// Blocks are the top-level structures like lines, code blocks, tables, etc.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<T> {
    /// The indentation level of the block.
    pub indent: usize,
    /// The actual content of the block.
    pub content: BlockContent<T>,
}

/// The content of a block-level element.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlockContent<T> {
    /// A normal line of text, composed of a sequence of inline nodes.
    Line(Vec<Node<T>>),

    /// A code block with optional filename and indentation.
    ///
    /// Starts with a `code:` prefix line; the body is the indented text that follows.
    CodeBlock {
        /// Filename and/or filetype metadata parsed from the `code:` prefix line.
        meta: CodeBlockMeta,
        /// The indentation level of the code block content.
        indent: usize,
        /// The raw content of the code block.
        content: String,
    },

    /// A table with a name and rows of cells.
    Table {
        /// The name of the table.
        name: String,
        /// The rows of the table, where each cell is a sequence of inline nodes.
        ///
        /// Structure: Rows -> Cells -> Content (Nodes)
        rows: Vec<Vec<Vec<Node<T>>>>,
    },

    /// A quote block, composed of a sequence of inline nodes.
    ///
    /// Content of quote is also subject to inline parsing.
    Quote(Vec<Node<T>>),

    /// A Helpfeel search-query line (starts with `? `).
    ///
    /// Content after `? ` is stored as a raw string; not parsed as inline nodes.
    Helpfeel(String),

    /// A command-line notation block (starts with `$ `).
    ///
    /// Content after `$ ` is stored as a raw string; not parsed as inline nodes.
    CommandLine(String),

    /// A custom block-level extension.
    ///
    /// This allows for extending the parser with custom block types (e.g., YouTube embeddings, special div blocks).
    Custom(T),
}

/// Metadata parsed from a `code:` prefix line.
///
/// # Examples
///
/// | Syntax | Variant |
/// |--------|---------|
/// | `code:` | `None` |
/// | `code:main.rs` | `Either("main.rs")` |
/// | `code:main.rs(rust)` | `Both { filename: "main.rs", filetype: "rust" }` |
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CodeBlockMeta {
    /// No filename or filetype specified (`code:`).
    None,
    /// Either a filename or a filetype, but not both (`code:main.rs` or `code:rust`).
    Either(String),
    /// Both filename and explicit filetype (`code:main.rs(rust)`).
    Both { filename: String, filetype: String },
}
