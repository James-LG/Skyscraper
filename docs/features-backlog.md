# Features Backlog

Remaining gaps and design decisions compared to the official W3C XPath 3.1 and WHATWG HTML specifications.
Last audited: 2026-03-02.

---

## XPath 3.1 — Axes

### Not Applicable

| Feature | Notes |
|---------|-------|
| `namespace::` axis | Parses, but returns error at eval time — intentional for HTML-only processor (no namespace nodes) | `src/xpath/grammar/expressions/path_expressions/steps/forward_step.rs:97` |

---

## XPath 3.1 — Node Tests

### Partially Implemented

| Feature | Gap | Location |
|---------|-----|----------|
| `namespace-node()` | Parses but always returns empty for HTML | `src/xpath/grammar/types/mod.rs:184-187` |

---

## XPath 3.1 — Type System

### Not Implemented

| Feature | Notes |
|---------|-------|
| `xs:decimal` | Would need arbitrary-precision arithmetic |
| `xs:date`, `xs:dateTime`, `xs:time` | No temporal types |
| `xs:duration`, `xs:yearMonthDuration`, `xs:dayTimeDuration` | No duration types |
| `xs:anyURI` | No URI atomic value type |
| `xs:untypedAtomic` | Not distinguished from string |
| `xs:hexBinary`, `xs:base64Binary` | No binary types |
| `xs:NOTATION` | Not applicable for HTML |

---

## XPath 3.1 — Built-in Functions

### Not Implemented — By Category

#### String (medium priority)
`fn:analyze-string` (requires XML node construction for result element)

#### QName functions (low priority — less relevant for HTML)
`fn:namespace-uri-for-prefix`, `fn:in-scope-prefixes`, `fn:resolve-QName`

#### Date/Time functions (low priority — require atomic type support first)
All `*-from-duration`, `*-from-dateTime`, `*-from-date`, `*-from-time`, `fn:current-dateTime`, `fn:current-date`, `fn:current-time`, `fn:format-dateTime`, `fn:format-date`, `fn:format-time`, `fn:adjust-*-to-timezone`, `fn:implicit-timezone`

#### Document functions (low priority — less relevant for in-memory HTML)
`fn:doc`, `fn:doc-available`, `fn:collection`, `fn:uri-collection`, `fn:unparsed-text`, `fn:unparsed-text-lines`, `fn:unparsed-text-available`, `fn:environment-variable`, `fn:available-environment-variables`

#### Parsing/Serialization (low priority)
`fn:parse-xml`, `fn:parse-xml-fragment`, `fn:serialize`, `fn:json-doc`, `fn:parse-json`, `fn:json-to-xml`, `fn:xml-to-json`

---

## HTML Parser — Design Decisions

| Feature | Decision | Rationale |
|---------|----------|-----------|
| `<template>` content model | Children are kept directly under the template element, not in a separate `DocumentFragment` | Skyscraper has no DOM API (`template.content`), inertness is already satisfied (no script execution), and direct children are more useful for XPath queries (e.g. `//template/div`). Same approach as html5ever's rcdom. |
| Implicit `<tbody>` insertion | Skyscraper inserts a `<tbody>` element when `<tr>`, `<td>`, or `<th>` appear as direct children of `<table>`, per WHATWG §13.2.6.4.9 ("in table" insertion mode). Python's lxml does **not** insert `<tbody>` in this case. This means `//table//*` returns one extra element (the implicit `<tbody>`) in Skyscraper compared to lxml. The lxml reference tests use explicit `<tbody>` in test HTML to keep both parsers aligned. | `src/html/grammar/insertion_mode_impls/mod.rs:1264-1272` |

## HTML Parser — Not Implemented / Intentionally Omitted

| Feature | Notes |
|---------|-------|
| Scripting | All script-related processing is intentionally omitted (prepare-the-script-element algorithm, script execution, `DOMContentLoaded` / `load` events). `<script>` content is correctly consumed via ScriptData tokenizer state but never executed. `<noscript>` is treated as if scripting is disabled (correct for a scraper). |
| Declarative Shadow DOM | `<template shadowrootmode="...">` is treated as a plain `<template>` | `src/html/grammar/insertion_mode_impls/mod.rs:317` |
| `<meta>` encoding change | Spec requires checking charset/http-equiv/content attributes to potentially change character encoding; no-op since Skyscraper operates on already-decoded Rust `char` data | `src/html/grammar/insertion_mode_impls/mod.rs:267` |

---

## XPath 3.1 — Expressions

### Known Limitations

| Feature | Gap | Location |
|---------|-----|----------|
| Inline function closure capture | Inline functions (`function($x) { $x + $y }`) do not capture variables from the definition scope. The body is stored as source text and re-parsed at call time using the caller's variable context, not the definition-time context. Fixing this requires adding a `'tree` lifetime parameter to the `Function` enum to store captured `XpathItemSet` bindings, which cascades to ~94 occurrences across 8 files. | `src/xpath/grammar/data_model/mod.rs:147-148`, `src/xpath/grammar/expressions/postfix_expressions.rs:280-305`, `src/xpath/grammar/expressions/primary_expressions/mod.rs:171-189` |

---

## Data Model Gaps

| Feature | Gap | Location |
|---------|-----|----------|
| Namespace nodes | Not represented in the tree at all | N/A |
