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
    /// The indentation level of the block is on the enclosing [`Block::indent`].
    CodeBlock {
        /// Filename and/or language metadata parsed from the `code:` prefix line.
        meta: CodeBlockMeta,
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

    /// A command-line notation block (starts with `$ ` or `% `).
    ///
    /// `prompt` records which prefix character was used; `command` is the
    /// raw text after the prefix (not parsed as inline nodes).
    CommandLine {
        /// The shell prompt character that introduced this command.
        prompt: ShellPrompt,
        /// The raw command text after the prompt and the trailing space.
        command: String,
    },

    /// A custom block-level extension.
    ///
    /// This allows for extending the parser with custom block types (e.g., YouTube embeddings, special div blocks).
    Custom(T),
}

/// The prompt character that introduced a [`BlockContent::CommandLine`].
///
/// Cosense supports two prompt styles for command-line notation:
/// `$` for Bourne shell-family prompts and `%` for csh/tcsh-family prompts.
/// Renderers may use this to display the original prompt character.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShellPrompt {
    /// `$` — Bourne shell-family prompt.
    Dollar,
    /// `%` — csh/tcsh-family prompt.
    Percent,
}

/// Metadata parsed from a `code:` prefix line.
///
/// Cosense's `code:` syntax does not distinguish between a filename and a
/// language identifier in the single-token form. When only one token is
/// present (e.g. `code:foo`), the variant [`CodeBlockMeta::NameOrLang`] is
/// produced and the consumer is responsible for any further classification
/// (e.g. by inspecting the file extension).
///
/// # Examples
///
/// | Syntax | Variant |
/// |--------|---------|
/// | `code:` | `None` |
/// | `code:main.rs` | `NameOrLang("main.rs")` |
/// | `code:rust` | `NameOrLang("rust")` |
/// | `code:main.rs(rust)` | `Both { filename: "main.rs", language: "rust" }` |
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CodeBlockMeta {
    /// No filename or language specified (`code:`).
    None,
    /// A single token whose role (filename or language) is not determined by
    /// the syntax. The `code:` form does not require a filename to have an
    /// extension, so a bare `code:foo` cannot be classified by the parser.
    NameOrLang(String),
    /// Both filename and language are explicitly specified
    /// (`code:main.rs(rust)`).
    Both {
        /// The filename portion (e.g., `"main.rs"`).
        filename: String,
        /// The language portion (e.g., `"rust"`).
        language: String,
    },
}
