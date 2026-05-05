# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.1.0] - 2026-04-26

### Added

- Initial release of the cosy parser library.
- Block-level parsing: indented lists, code blocks (`code:`), tables (`table:`),
  quotes (`>`), Helpfeel (`? `), and command-line notation (`$ ` / `% `).
- Inline parsing: internal/external links, cross-project links, labeled links,
  images (MIME-detected), linked images, icons with repeat count (`[name.icon*N]`),
  inline code, math (`[$ expr]`), decorations (`[* bold]`, `[/ italic]`, etc.),
  and hashtags (`#tag`).
- All URL-valued AST fields are stored as [`url::Url`]: `Node::Image`,
  `Node::LinkedImage.src` / `.href`, `Link::Url`, and `Link::WithLabel.href`.
- `[[text]]` strong (large bold) syntax, including `[[image]]`, `[[url]]`, and
  `[[name.icon]]` variants.
- `[N35.xx,E139.xx]` geographic coordinate syntax with optional zoom level
  (`[N35.xx,E139.xx,Z14]`), parsed as `Node::Coordinate` with `f64` latitude
  and longitude.
- `CosyParserExtension` trait with a `parse_bracket` hook for user-defined
  bracket syntax extensions.
- `ShellPrompt { Dollar, Percent }` enum on `BlockContent::CommandLine` so
  renderers can recover the original prompt character.
- `Document<T>` newtype around `Vec<Block<T>>` with `Deref<Target = [Block<T>]>`
  and `IntoIterator` impls.
- `cosy::Url` re-export of [`url::Url`] for convenience.
- Optional `serde` feature for serialization/deserialization of AST types
  (also enables `url/serde`).
- `ParseError` type (using `thiserror`) so callers do not need a direct
  `winnow` dependency.
