//! Inline-level AST nodes.

// --------------------------------------------------------
// Inline level (character-based structure)
// --------------------------------------------------------

/// Represents an inline-level element (node) within a block.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub enum Node<T> {
    /// Plain text.
    Text(String),

    /// A link (internal or external).
    ///
    /// Requires `T` because the label contains `Node`s.
    Link(Link<T>),

    /// An image reference.
    Image(url::Url),

    /// An image with a clickable link.
    LinkedImage {
        /// URL of the image to display.
        src: url::Url,
        /// URL to navigate to when clicked.
        href: url::Url,
    },

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

    /// A hashtag reference (e.g., `#タグ名`).
    Hashtag(String),

    /// Decorated text (bold, italic, etc.).
    ///
    /// Requires `T` due to recursive structure.
    Decoration {
        /// The decoration characters (e.g., "*", "*-").
        decos: String,
        /// The content inside the decoration.
        nodes: Vec<Node<T>>,
    },

    /// Strong (large bold) content rendered by `[[...]]`.
    ///
    /// The inner content is parsed as inline nodes, so nested syntax
    /// such as links and hashtags is supported.
    Strong(Vec<Node<T>>),

    /// A geographic coordinate rendered as a map embed.
    ///
    /// Parsed from `[N35.65,E139.74]` or `[N35.65,E139.74,Z14]`.
    Coordinate {
        /// Latitude value (e.g., `35.6578589`).
        lat: f64,
        /// Latitude hemisphere.
        lat_dir: LatitudeDirection,
        /// Longitude value (e.g., `139.7474797`).
        lon: f64,
        /// Longitude hemisphere.
        lon_dir: LongitudeDirection,
        /// Optional zoom level (e.g., `Some(14)`).
        zoom: Option<u32>,
    },

    /// A custom inline-level extension.
    ///
    /// This allows for extending the parser with custom inline types (e.g., colored text, warning badges).
    Custom(T),
}

/// Hemisphere of a latitude value in [`Node::Coordinate`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatitudeDirection {
    /// Northern hemisphere (`N` prefix).
    North,
    /// Southern hemisphere (`S` prefix).
    South,
}

/// Hemisphere of a longitude value in [`Node::Coordinate`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LongitudeDirection {
    /// Eastern hemisphere (`E` prefix).
    East,
    /// Western hemisphere (`W` prefix).
    West,
}

/// Represents a link target and optional label.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub enum Link<T> {
    /// A link to another page (internal link).
    Page(String),
    /// A raw URL (external link).
    Url(url::Url),
    /// A link with an explicit label.
    ///
    /// `WithLabel` is only produced when one of the bracket tokens parses as
    /// a URL, so `href` is always a valid [`url::Url`].
    WithLabel {
        /// The destination URL.
        href: url::Url,
        /// The label content, which can contain other inline nodes.
        ///
        /// The label might contain `Custom` nodes.
        label: Vec<Node<T>>,
    },

    /// A cross-project link to a project's home: [/project] or [/project/]
    Project(String),

    /// A cross-project page link: [/project/page]
    ProjectPage {
        /// The target project name.
        project: String,
        /// The target page name within the project.
        page: String,
    },
}
