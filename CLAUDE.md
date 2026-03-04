# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Skyscraper is a Rust library for scraping HTML documents with XPath expressions. It has its own HTML parser (following the WHATWG spec) and XPath parser/evaluator. Published on crates.io as `skyscraper` (v0.7.0-beta.2, MIT license). Many XPath features still have `todo!()` stubs.

## Build & Test Commands

```sh
cargo build                  # Build
cargo test                   # Run all tests
cargo test <test_name>       # Run a single test by name
cargo test --test <file>     # Run a specific test file (e.g. --test html_tests)
cargo bench                  # Run all benchmarks (criterion)
```

Stack overflow regression tests (Windows-specific, run in CI on windows-latest):
```sh
cargo test --test run_stack_overflow_tests -- --include-ignored --test-threads=1
```

Reference testing against Python lxml (managed via uv):
```sh
cd tests/lxml_tests && uv sync          # Install Python deps (first time / after changes)
cat tests/samples/James-LG_Skyscraper.html | uv run --directory tests/lxml_tests python xpath.py "//div"
```
The Rust lxml tests (`tests/xpath_tests/lxml_tests.rs`) automatically use the venv at `tests/lxml_tests/.venv/bin/python` when available.

Feature flag: `debug_prints` enables debug output during parsing.

## Architecture

The crate has two core modules (`src/html/` and `src/xpath/`) plus a small `vecpointer` utility. The `#![warn(missing_docs)]` lint is enabled at the crate root.

### HTML Module (`src/html/`)

Parses HTML text into an arena-based tree (`indextree` crate).

- **`HtmlDocument`** — owns the arena tree; has a `root_node: DocumentNode`
- **`DocumentNode`** — wrapper around `indextree::NodeId`; provides `children()`, `parent()`, `get_text()`, `get_all_text()`
- **`HtmlNode`** — enum: `Tag(HtmlTag)` | `Text(HtmlText)`. Text nodes are separate to preserve ordering in mixed content.
- **Grammar subsystem** (`src/html/grammar/`):
  - **Tokenizer** (`tokenizer/`) — state-machine HTML tokenizer per WHATWG spec, including named character references
  - **Parser** (`mod.rs`) — main parsing logic with WHATWG insertion modes
  - **Document builder** (`document_builder.rs`) — constructs the tree from token stream
  - **Insertion modes** (`insertion_mode_impls/`) — WHATWG tree construction insertion modes (in_body is the largest)
- Key constants: `VOID_TAGS`, `SPECIAL_ELEMENTS`, `GENERATE_IMPLIED_END_TAG_TYPES`, `ELEMENT_IN_SCOPE_TYPES` (defined via `once_cell::Lazy`)

### XPath Module (`src/xpath/`)

Parses XPath strings and evaluates them against an `XpathItemTree` (converted from `HtmlDocument`).

- **`Xpath`** — compiled XPath expression; call `.apply(&XpathItemTree)` to evaluate
- **`XpathItemTree`** — tree structure mirroring the HTML document for XPath evaluation; created via `XpathItemTree::from(&document)`
- **Grammar subsystem** (`src/xpath/grammar/`, ~70 files):
  - `expressions/` — all expression types (path, logical, comparison, arithmetic, conditional, for/let, quantified, postfix, etc.)
  - `expressions/path_expressions/steps/axes/` — forward axes (child, descendant, attribute, descendant-or-self) and reverse axes (parent)
  - `data_model/` — XPath node types (document, element, text, attribute, comment, PI)
  - `types/` — XPath type system (sequence types, element/attribute tests)
  - `terminal_symbols.rs` — token definitions
  - `recipes.rs`, `whitespace_recipes.rs` — nom parser combinator helpers
  - `xml_names.rs` — XML name validation

### Parsing Approach

Both modules use **nom** parser combinators extensively. The `recipes.rs` and `whitespace_recipes.rs` modules provide reusable combinator patterns. The HTML tokenizer uses a state machine pattern, while XPath parsing is a recursive descent grammar built from nom combinators.

## Test Structure

- `tests/html_tests/` — HTML parsing correctness
- `tests/xpath_tests/` — XPath parsing and evaluation (many sub-files: predicates, axes, functions, type matching, etc.)
- `tests/test_framework/` — shared test utilities for document comparison
- `tests/samples/` — sample HTML files for integration tests
- `tests/lxml_tests/` — Python reference tests comparing against lxml
- `tests/stack_overflow_tests/` — separate Cargo project for Windows stack overflow regression

Tests use `indoc` for readable multi-line HTML strings and `proptest` for property-based testing.

## Reference Specifications (`.context/`)

The `.context/` directory (gitignored) contains downloaded W3C specifications for offline reference. New resources can be added here as needed to avoid refetching.

Current contents:
- `xpath-31-spec.html` — [XPath 3.1](https://www.w3.org/TR/2017/REC-xpath-31-20170321/) (W3C Recommendation)
- `xpath-datamodel-31-spec.html` — [XPath Data Model 3.1](https://www.w3.org/TR/2017/REC-xpath-datamodel-31-20170321/) (W3C Recommendation)
- `xpath-functions-31-spec.html` — [XPath Functions and Operators 3.1](https://www.w3.org/TR/2017/REC-xpath-functions-31-20170321/) (W3C Recommendation)

## Features Backlog

[`docs/features-backlog.md`](docs/features-backlog.md) tracks spec gaps, known limitations, and design decisions for both the HTML parser and XPath evaluator. Check it before implementing new features to avoid duplicating known issues.

## CI

GitHub Actions (`.github/workflows/rust.yml`): runs `cargo test` in a devcontainer on ubuntu-latest, plus stack overflow tests on windows-latest with Rust nightly.
