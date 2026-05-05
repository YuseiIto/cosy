# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**cosy** is a Rust library that parses Cosense/Scrapbox markup syntax into a typed AST. It uses the `winnow` parser combinator library and supports user-defined syntax extensions via a generic trait.

## Commands

```bash
cargo build                  # Build
cargo test                   # Run all tests
cargo test --lib             # Library tests only
cargo test <test_name>       # Single test (e.g. cargo test test_parse_deco)
cargo fmt --all -- --check   # Check formatting
cargo clippy                 # Lint

# Examples
cargo run --example basic
cargo run --example speech_bubble_extension
cargo run --example cli -- local <path>
cargo run --example cli -- title <project>/<page> [api-key]
```

## Architecture

### Two-Level AST

- **Block level** (`ast/block.rs`): `Document<T>` is a newtype `Document<T>(pub Vec<Block<T>>)` with `Deref<Target = [Block<T>]>` and `IntoIterator` impls. `BlockContent<T>` variants: Line, CodeBlock, Table, Quote, Helpfeel, CommandLine, Custom. Each block tracks indentation level.
- **Inline level** (`ast/node.rs`): `Node<T>` variants: Text, Link, Image, LinkedImage, Icon, InlineCode, Math, Hashtag, Decoration, Strong, Coordinate, Custom. `Link<T>` further splits into Page, Url, WithLabel, Project, ProjectPage.

All AST types are generic over `T` — the extension output type.

### Parsing Flow

`parse()` → `parse_block()` (dispatches by block type) → for lines, `parse_nodes()` tries inline parsers in priority order via `winnow::combinator::alt`:

1. Backtick inline code
2. `[$ math]` inline math
3. Custom bracket extensions
4. `[* decorations]` (with recursive inner node parsing)
5. Standard `[bracket]` links
6. Plain text fallback

Bracket content and decorations recursively call `parse_nodes()`, enabling nested structures like `[* [Link] text]`.

### Extension System

`CosyParserExtension` trait (`extension.rs`) lets consumers define custom syntax:

```rust
pub trait CosyParserExtension {
    type Output;
    fn parse_bracket(&self, content: &str) -> Option<Self::Output>;
}
```

`()` implements the trait as a no-op default. Return `None` to fall through to built-in parsers. See `examples/speech_bubble_extension.rs` for usage.

### Key Modules

- `parser/` — All parsing logic, one file per syntax element
- `ast/` — Type definitions only, no logic
- `tokens.rs` — Shared string constants (delimiters, prefix markers, decoration chars)
- `url.rs` — URL detection and MIME type inference for distinguishing link vs image
