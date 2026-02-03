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
#[derive(Debug, PartialEq, Clone)]
pub struct Block<T> {
    /// The indentation level of the block.
    pub indent: usize,
    /// The actual content of the block.
    pub content: BlockContent<T>,
}

/// The content of a block-level element.
#[derive(Debug, PartialEq, Clone)]
pub enum BlockContent<T> {
    /// A normal line of text, composed of a sequence of inline nodes.
    Line(Vec<Node<T>>),

    /// A code block with optional filename and indentation.
    CodeBlock {
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

    /// A custom block-level extension.
    ///
    /// This allows for extending the parser with custom block types (e.g., YouTube embeddings, special div blocks).
    Custom(T),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CodeBlockMeta {
    None,
    Either(String),
    Both { filename: String, filetype: String },
}
