# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-26

### Added

- Initial release of the cosy parser library.
- Block-level parsing: indented lists, code blocks (`code:`), tables (`table:`),
  quotes (`>`), Helpfeel (`? `), and command-line notation (`$ `).
- Inline parsing: internal/external links, cross-project links, labeled links,
  images (MIME-detected), linked images, icons with repeat count (`[name.icon*N]`),
  inline code, math (`[$ expr]`), decorations (`[* bold]`, `[/ italic]`, etc.),
  and hashtags (`#tag`).
- `CosyParserExtension` trait for user-defined bracket and block syntax extensions.
- Optional `serde` feature for serialization/deserialization of AST types.
- `ParseError` type (using `thiserror`) so callers do not need a direct `winnow`
  dependency.
