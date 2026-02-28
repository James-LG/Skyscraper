# Features Backlog

Comparison of the current codebase against the official W3C XPath 3.1 and WHATWG HTML specifications.
Last audited: 2026-02-27.

---

## XPath 3.1 — Expressions and Operators

### Fully Implemented

- Primary expressions: integer/decimal/double/string literals, variable references, parenthesized expressions, context item (`.`), function calls, named function references, inline functions, map constructors, array constructors (square and curly), unary lookups
- Path expressions: `/`, `//`, relative paths, step expressions, abbreviated syntax (`@`, `..`)
- Comparison expressions: value (`eq ne lt le gt ge`), general (`= != < <= > >=`), node (`is << >>`)
- Logical expressions: `and`, `or`
- Arithmetic expressions: `+ - * div idiv mod`, unary `+/-`
- String concatenation: `||`
- Range expressions: `to`
- Conditional expressions: `if/then/else`
- For expressions: `for $var in seq return expr` (multiple bindings)
- Let expressions: `let $var := expr return expr` (multiple bindings)
- Quantified expressions: `some`, `every` (multiple bindings)
- Postfix expressions: predicates `[expr]`, argument lists, lookup `?key`
- Simple map operator: `!`
- Sequence construction: comma operator
- Set operations: `union` / `|`, `intersect`, `except` (with document ordering)
- Maps and arrays: constructors, lookup via `?`, wildcard lookups, parenthesized key expressions
- `instance of`, `castable as`, `treat as`
- Partial function application via argument placeholders (`?`)

### Partially Implemented

| Feature | Gap | Location |
|---------|-----|----------|
| ~~Arrow operator (`=>`)~~ | ~~Done~~ — VarRef and ParenthesizedExpr function specifiers now supported | `src/xpath/grammar/expressions/arrow_operator.rs` |
| ~~`cast as`~~ | ~~Done~~ — URI-qualified type names (`Q{uri}type`) now supported for `xs` namespace | `src/xpath/grammar/expressions/expressions_on_sequence_types/cast.rs:113` |
| ~~Item sorting~~ | ~~Done~~ — Stale TODO removed; sorting is already handled at the correct level (path expressions and set operations) | `src/xpath/grammar/expressions/mod.rs` |

---

## XPath 3.1 — Axes

### Fully Implemented

All 13 axes parse and evaluate:

**Forward:** `child`, `descendant`, `attribute`, `self`, `descendant-or-self`, `following-sibling`, `following`
**Reverse:** `parent`, `ancestor`, `preceding-sibling`, `preceding`, `ancestor-or-self`

### Partially Implemented

| Feature | Gap | Location |
|---------|-----|----------|
| `namespace::` axis | Parses, but returns error at eval time ("not supported for HTML documents") — intentional for HTML-only processor | `src/xpath/grammar/expressions/path_expressions/steps/forward_step.rs:97` |
| ~~`parent::` on attributes~~ | ~~Done~~ — Already works via arena tree; stale TODO removed, test added | `src/xpath/grammar/expressions/path_expressions/steps/reverse_step.rs:115` |

---

## XPath 3.1 — Node Tests

### Fully Implemented

- Name tests: QName, `*`, `*:localname`, `prefix:*`, `Q{uri}name`
- Kind tests: `node()`, `text()`, `comment()`, `element()`, `element(name)`, `element(*)`, `attribute()`, `attribute(name)`, `attribute(*)`, `item()`, `document-node()`, `document-node(element-test)`, `processing-instruction()`, `processing-instruction(name)`
- Schema-aware tests (correctly return empty for non-schema-aware processor): `schema-element()`, `schema-attribute()`
- Function/map/array tests: `function(*)`, `map(*)`, `array(*)`

### Partially Implemented

| Feature | Gap | Location |
|---------|-----|----------|
| `namespace-node()` | Parses but always returns empty for HTML | `src/xpath/grammar/types/mod.rs:184-187` |
| ~~Typed function/map/array tests~~ | ~~Done~~ — `map(K,V)` validates key/value types, `array(T)` validates member types, `function(T1,...) as R` validates arity; used by `instance of` and `treat as` | `src/xpath/grammar/types/sequence_type.rs` |

---

## XPath 3.1 — Type System

### Fully Implemented

- Atomic types: `xs:integer` (i64), `xs:string`, `xs:boolean`, `xs:double` (f64), `xs:float` (f32)
- Occurrence indicators: `?`, `*`, `+`
- `empty-sequence()` matching
- Cast rules between the 5 implemented atomic types

### Not Implemented

| Feature | Notes |
|---------|-------|
| `xs:decimal` | Would need arbitrary-precision arithmetic |
| `xs:date`, `xs:dateTime`, `xs:time` | No temporal types |
| `xs:duration`, `xs:yearMonthDuration`, `xs:dayTimeDuration` | No duration types |
| `xs:QName` | No QName atomic value type |
| `xs:anyURI` | No URI atomic value type |
| `xs:untypedAtomic` | Not distinguished from string |
| `xs:hexBinary`, `xs:base64Binary` | No binary types |
| `xs:NOTATION` | Not applicable for HTML |

---

## XPath 3.1 — Built-in Functions

**92** `fn:` functions plus **10** `map:`, **18** `array:`, and **14** `math:` functions (**134 total**) are implemented. All dispatch is in `src/xpath/grammar/expressions/primary_expressions/static_function_calls.rs`.

### Implemented

| Function | Status | Notes |
|----------|--------|-------|
| `fn:root()` | Complete | Special-cased early in eval; always returns document root |
| `fn:contains(string, string)` | Complete | |
| `fn:data(item*)` | Complete | Raises `err:FOTY0013` for function items |
| `fn:string(item?)` | Complete | Raises `err:FOTY0014` for function items |
| `fn:true()` | Complete | |
| `fn:false()` | Complete | |
| `fn:not(item*)` | Complete | |
| `fn:boolean(item*)` | Complete | |
| `fn:number(item?)` | Complete | |
| `fn:abs(numeric)` | Complete | |
| `fn:ceiling(numeric)` | Complete | |
| `fn:floor(numeric)` | Complete | |
| `fn:round(numeric)` | Complete | |
| `fn:concat(atomic, atomic, ...)` | Complete | |
| `fn:string-join(string*, string?)` | Complete | |
| `fn:string-length(string?)` | Complete | |
| `fn:normalize-space(string?)` | Complete | |
| `fn:upper-case(string)` | Complete | |
| `fn:lower-case(string)` | Complete | |
| `fn:starts-with(string, string)` | Complete | |
| `fn:ends-with(string, string)` | Complete | |
| `fn:substring(string, double, double?)` | Complete | |
| `fn:substring-before(string, string)` | Complete | |
| `fn:substring-after(string, string)` | Complete | |
| `fn:translate(string, string, string)` | Complete | |
| `fn:empty(item*)` | Complete | |
| `fn:exists(item*)` | Complete | |
| `fn:count(item*)` | Complete | |
| `fn:head(item*)` | Complete | |
| `fn:tail(item*)` | Complete | |
| `fn:reverse(item*)` | Complete | |
| `fn:distinct-values(item*)` | Complete | |
| `fn:sum(item*, item?)` | Complete | |
| `fn:name(node?)` | Complete | |
| `fn:local-name(node?)` | Complete | Identical to `fn:name` in HTML-only processor |
| `fn:position()` | Complete | |
| `fn:last()` | Complete | |
| `fn:matches(string, string, string?)` | Complete | Regex-based; supports `i`, `s`, `m`, `x` flags |
| `fn:replace(string, string, string, string?)` | Complete | Regex-based; XPath `$N` backreferences compatible with Rust regex |
| `fn:tokenize(string, string?, string?)` | Complete | 1-arg whitespace form and regex form |
| `fn:subsequence(item*, double, double?)` | Complete | |
| `fn:insert-before(item*, integer, item*)` | Complete | |
| `fn:remove(item*, integer)` | Complete | |
| `fn:index-of(item*, item)` | Complete | |
| `fn:zero-or-one(item*)` | Complete | Cardinality assertion |
| `fn:one-or-more(item*)` | Complete | Cardinality assertion |
| `fn:exactly-one(item*)` | Complete | Cardinality assertion |
| `fn:avg(item*)` | Complete | Returns `xs:double` |
| `fn:max(item*)` | Complete | Numeric and string comparison |
| `fn:min(item*)` | Complete | Numeric and string comparison |
| `fn:round-half-to-even(numeric, integer?)` | Complete | Banker's rounding via `round_ties_even()` |
| `fn:format-integer(integer, string)` | Complete | Supports `1`, `01`, `a`/`A`, `i`/`I`, `w`/`W` picture strings |
| `fn:compare(string, string)` | Complete | Returns -1, 0, or 1 |
| `fn:codepoint-equal(string, string)` | Complete | |
| `fn:codepoints-to-string(integer*)` | Complete | |
| `fn:string-to-codepoints(string)` | Complete | |
| `fn:encode-for-uri(string)` | Complete | RFC 3986 percent-encoding |
| `fn:iri-to-uri(string)` | Complete | Encodes non-ASCII and disallowed URI characters |
| `fn:escape-html-uri(string)` | Complete | Encodes characters outside printable ASCII |
| `fn:deep-equal(item*, item*)` | Complete | Positional comparison using `PartialEq` |
| `fn:unordered(item*)` | Complete | Identity function (optimization hint) |
| `fn:has-children(node?)` | Complete | |
| `fn:path(node?)` | Complete | Returns XPath path expression with positional predicates |
| `fn:namespace-uri(node?)` | Complete | Always returns empty string (HTML-only processor) |
| `fn:lang(string)` | Complete | Walks up ancestors looking for `lang`/`xml:lang` attribute |
| `fn:node-name(node?)` | Complete | Returns element/attribute name |
| `fn:nilled(node?)` | Complete | Always returns false (HTML-only processor) |
| `fn:generate-id(node?)` | Complete | Returns `N{node_id}` |
| `fn:for-each(item*, function)` | Complete | Higher-order: applies function to each item |
| `fn:filter(item*, function)` | Complete | Higher-order: keeps items where function returns true |
| `fn:fold-left(item*, item, function)` | Complete | Higher-order: left fold with accumulator |
| `fn:fold-right(item*, item, function)` | Complete | Higher-order: right fold with accumulator |
| `fn:for-each-pair(item*, item*, function)` | Complete | Higher-order: pairwise application |
| `fn:sort(item*, string?, function?)` | Complete | 1-3 arg form; collation ignored, key function supported |
| `fn:apply(function, array)` | Complete | Invokes function with array items as arguments |
| `fn:function-name(function)` | Complete | Returns name for named functions, empty for anonymous |
| `fn:function-arity(function)` | Complete | Returns arity for all function item types |
| `fn:format-number(value, picture, name?)` | Complete | Picture string parsing; decimal-format-name ignored |
| `fn:normalize-unicode(string, form?)` | Complete | NFC, NFD, NFKC, NFKD via `unicode-normalization` crate |
| `fn:innermost(nodes)` | Complete | Filters to nodes with no descendant in the set |
| `fn:outermost(nodes)` | Complete | Filters to nodes with no ancestor in the set |
| `fn:base-uri(node?)` | Complete | Returns empty string (HTML-only processor) |
| `fn:document-uri(node?)` | Complete | Returns empty sequence (HTML-only processor) |
| `fn:error(code?, description?, object?)` | Complete | 0-3 args; default code `err:FOER0000` |
| `fn:trace(value, label?)` | Complete | Logs to stderr; returns input unchanged |
| `fn:id(string*, node?)` | Complete | Tokenizes on whitespace; finds elements by `id` attribute; document order |
| `fn:element-with-id(string*, node?)` | Complete | Identical to `fn:id` for non-schema-aware (HTML) processor |
| `fn:idref(string*, node?)` | Complete | Always returns empty sequence (no `is-idrefs` in HTML) |

### Implemented — Map Functions

| Function | Status | Notes |
|----------|--------|-------|
| `map:size(map)` | Complete | |
| `map:keys(map)` | Complete | |
| `map:contains(map, key)` | Complete | |
| `map:get(map, key)` | Complete | |
| `map:put(map, key, value)` | Complete | |
| `map:entry(key, value)` | Complete | |
| `map:remove(map, keys)` | Complete | |
| `map:merge(maps, options?)` | Complete | Options arg accepted but ignored; default first-wins policy |
| `map:for-each(map, function)` | Complete | Higher-order |
| `map:find(input, key)` | Complete | Recursive search into nested maps/arrays |

### Implemented — Array Functions

| Function | Status | Notes |
|----------|--------|-------|
| `array:size(array)` | Complete | |
| `array:get(array, position)` | Complete | |
| `array:put(array, position, value)` | Complete | |
| `array:append(array, value)` | Complete | |
| `array:subarray(array, start, length?)` | Complete | |
| `array:remove(array, positions)` | Complete | |
| `array:insert-before(array, position, value)` | Complete | |
| `array:head(array)` | Complete | |
| `array:tail(array)` | Complete | |
| `array:reverse(array)` | Complete | |
| `array:join(arrays)` | Complete | |
| `array:flatten(input)` | Complete | Recursive flattening |
| `array:for-each(array, function)` | Complete | Higher-order |
| `array:filter(array, function)` | Complete | Higher-order |
| `array:fold-left(array, zero, function)` | Complete | Higher-order |
| `array:fold-right(array, zero, function)` | Complete | Higher-order |
| `array:for-each-pair(array1, array2, function)` | Complete | Higher-order |
| `array:sort(array, collation?, key?)` | Complete | Collation ignored; key function supported |

### Implemented — Math Functions

| Function | Status | Notes |
|----------|--------|-------|
| `math:pi()` | Complete | |
| `math:exp(value)` | Complete | |
| `math:exp10(value)` | Complete | |
| `math:log(value)` | Complete | |
| `math:log10(value)` | Complete | |
| `math:sqrt(value)` | Complete | |
| `math:sin(value)` | Complete | |
| `math:cos(value)` | Complete | |
| `math:tan(value)` | Complete | |
| `math:asin(value)` | Complete | |
| `math:acos(value)` | Complete | |
| `math:atan(value)` | Complete | |
| `math:pow(x, y)` | Complete | |
| `math:atan2(y, x)` | Complete | |

### Not Implemented — By Category

#### String (medium priority)
`fn:analyze-string` (requires XML node construction for result element)

#### Higher-order (medium priority)
`fn:function-lookup` (requires `xs:QName` atomic type support)

#### QName functions (low priority — less relevant for HTML)
`fn:QName`, `fn:prefix-from-QName`, `fn:local-name-from-QName`, `fn:namespace-uri-from-QName`, `fn:namespace-uri-for-prefix`, `fn:in-scope-prefixes`, `fn:resolve-QName`

#### Date/Time functions (low priority — require atomic type support first)
All `*-from-duration`, `*-from-dateTime`, `*-from-date`, `*-from-time`, `fn:current-dateTime`, `fn:current-date`, `fn:current-time`, `fn:format-dateTime`, `fn:format-date`, `fn:format-time`, `fn:adjust-*-to-timezone`, `fn:implicit-timezone`

#### Document functions (low priority — less relevant for in-memory HTML)
`fn:doc`, `fn:doc-available`, `fn:collection`, `fn:uri-collection`, `fn:unparsed-text`, `fn:unparsed-text-lines`, `fn:unparsed-text-available`, `fn:environment-variable`, `fn:available-environment-variables`

#### Parsing/Serialization (low priority)
`fn:parse-xml`, `fn:parse-xml-fragment`, `fn:serialize`, `fn:json-doc`, `fn:parse-json`, `fn:json-to-xml`, `fn:xml-to-json`

#### ~~ID functions (low priority)~~ — Done
~~`fn:id`, `fn:idref`, `fn:element-with-id`~~

---

## HTML Parser — WHATWG Compliance

### Fully Implemented

All 23 WHATWG tree construction insertion modes are present:
Initial, BeforeHtml, BeforeHead, InHead, InHeadNoscript, AfterHead, InBody, Text, InTable, InTableText, InCaption, InColumnGroup, InTableBody, InRow, InCell, InSelect, InSelectInTable, InTemplate, AfterBody, InFrameset, AfterFrameset, AfterAfterBody, AfterAfterFrameset

The tokenizer implements the WHATWG state machine including named character references.

### Partially Implemented / `todo!()` Stubs

| Feature | Gap | Location |
|---------|-----|----------|
| ~~Text insertion mode — EOF handling~~ | ~~Done~~ — parse error, pop node, switch to original mode, reprocess EOF | `src/html/grammar/insertion_mode_impls/mod.rs:556-573` |
| ~~AfterBody — Comment token~~ | ~~Done~~ — insert as last child of `<html>` element | `src/html/grammar/insertion_mode_impls/mod.rs:716-722` |
| ~~AfterBody — DocType token~~ | ~~Done~~ — parse error, ignore | `src/html/grammar/insertion_mode_impls/mod.rs:723-727` |
| ~~AfterAfterBody — Comment token~~ | ~~Done~~ — insert as last child of Document | `src/html/grammar/insertion_mode_impls/mod.rs:763-768` |
| Declarative Shadow DOM | Not supported (noted in comment) | `src/html/grammar/insertion_mode_impls/mod.rs:317` |
| ~~HTML fragment parsing algorithm~~ | ~~Done~~ — `parse_fragment()` implements WHATWG 13.4 | `src/html/grammar/mod.rs` |

### Partially Implemented — Foreign Content

| Feature | Status | Notes |
|---------|--------|-------|
| ~~Namespace assignment~~ | ~~Done~~ — `create_element` now sets namespace on MathML/SVG elements | `src/html/grammar/mod.rs` |
| ~~Adjust MathML attributes~~ | ~~Done~~ — `definitionurl` → `definitionURL` per WHATWG 13.2.6.1 | `src/html/grammar/mod.rs` |
| ~~Adjust SVG attributes~~ | ~~Done~~ — 62 camelCase attribute name corrections per WHATWG 13.2.6.2 | `src/html/grammar/mod.rs` |
| ~~Adjust foreign attributes~~ | ~~Done~~ — `Attribute` and `AttributeNode` structs now carry `namespace: Option<String>`; `adjust_foreign_attributes` sets xlink, xml, and xmlns namespaces per WHATWG 13.2.6.3 | `src/html/grammar/mod.rs` |
| ~~Foreign content parsing mode~~ | ~~Done~~ — tree construction dispatcher + full token processing rules per WHATWG 13.2.6.5; breakout tags, MathML text integration points, HTML integration points (SVG foreignObject/desc/title, MathML annotation-xml with encoding), end-tag walk-up algorithm | `src/html/grammar/insertion_mode_impls/in_foreign_content.rs` |
| ~~SVG element name case correction~~ | ~~Done~~ — 37 camelCase element name corrections (e.g. `foreignobject` → `foreignObject`) applied in `create_an_element_for_the_token` for SVG namespace | `src/html/grammar/mod.rs` |

### Not Implemented

| Feature | Notes |
|---------|-------|
| ~~Adoption agency algorithm edge cases~~ | ~~Done~~ — Step 4.6 now performs a proper scope check via `has_node_in_scope` (`src/html/grammar/insertion_mode_impls/in_body_insertion_mode.rs:1241`); formatting elements behind scope barriers (e.g. `table`) are correctly ignored per WHATWG spec |
| ~~Foster parenting~~ | ~~Done~~ — Verified with comprehensive edge-case tests: element/text/mixed foster parenting, multiple foster-parented elements, foster parenting after table rows, anchor elements, bold text; all pass correctly |
| ~~Duplicate attribute deduplication~~ | ~~Done~~ — `emit_current_tag_token` now deduplicates attributes per WHATWG 13.2.5.34, keeping the first occurrence and emitting a `DuplicateAttribute` parse error for subsequent duplicates (`src/html/grammar/tokenizer/mod.rs`) |

---

## Data Model Gaps

| Feature | Gap | Location |
|---------|-----|----------|
| Namespace nodes | Not represented in the tree at all | N/A |

---

## Error Handling Gaps

All previously identified error handling gaps have been resolved:
- `fn:data()` now raises `err:FOTY0013` for function items
- `fn:string()` now raises `err:FOTY0014` for function items
- Unknown function dispatch now uses `err:XPST0017`
