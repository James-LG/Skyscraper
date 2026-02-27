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
| Arrow operator (`=>`) | VarRef and ParenthesizedExpr function specifiers not supported — only static function call specifiers work | `src/xpath/grammar/expressions/arrow_operator.rs:104,111` |
| `cast as` | URI-qualified type names (`Q{uri}type`) not supported | `src/xpath/grammar/expressions/expressions_on_sequence_types/cast.rs:113` |
| Item sorting | Disabled with TODO comment — may affect document-order guarantees in some edge cases | `src/xpath/grammar/expressions/mod.rs:123` |

---

## XPath 3.1 — Axes

### Fully Implemented

All 13 axes parse and evaluate:

**Forward:** `child`, `descendant`, `attribute`, `self`, `descendant-or-self`, `following-sibling`, `following`
**Reverse:** `parent`, `ancestor`, `preceding-sibling`, `preceding`, `ancestor-or-self`

### Partially Implemented

| Feature | Gap | Location |
|---------|-----|----------|
| `namespace::` axis | Parses, but returns error at eval time ("not supported for HTML documents") | `src/xpath/grammar/expressions/path_expressions/steps/forward_step.rs:97` |
| `parent::` on attributes | Attribute nodes don't link back to their parent element | `src/xpath/grammar/expressions/path_expressions/steps/reverse_step.rs:116` |

---

## XPath 3.1 — Node Tests

### Fully Implemented

- Name tests: QName, `*`, `*:localname`, `prefix:*`, `Q{uri}name`
- Kind tests: `node()`, `text()`, `comment()`, `element()`, `element(name)`, `element(*)`, `attribute()`, `attribute(name)`, `attribute(*)`, `item()`
- Schema-aware tests (correctly return empty for non-schema-aware processor): `schema-element()`, `schema-attribute()`
- Function/map/array tests: `function(*)`, `map(*)`, `array(*)`

### Partially Implemented

| Feature | Gap | Location |
|---------|-----|----------|
| `document-node(element-test)` | Parametrized form parses but matches any document node (superset) | `src/xpath/grammar/types/mod.rs:295-300` |
| `processing-instruction(name)` | Parses, but `PINode` struct is empty — no target field to match against | `src/xpath/grammar/data_model/mod.rs:723` |
| `namespace-node()` | Parses but always returns empty for HTML | `src/xpath/grammar/types/mod.rs:184-187` |
| Typed function/map/array tests | `function(T as S)`, `map(K,V)`, `array(T)` parse but parameter/return types are not validated at runtime | `src/xpath/grammar/types/sequence_type.rs:211-213` |

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

Only **4** of the 120+ standard functions are implemented. All dispatch is in `src/xpath/grammar/expressions/primary_expressions/static_function_calls.rs`.

### Implemented

| Function | Status | Notes |
|----------|--------|-------|
| `fn:root()` | Complete | Special-cased early in eval; always returns document root |
| `fn:contains(string, string)` | Complete | |
| `fn:data(item*)` | Complete | TODO: should raise `err:FOTY0013` for function items |
| `fn:string(item?)` | Complete | TODO: should raise error for function items per spec |

### Not Implemented — By Category

#### Boolean (high priority — commonly used)
`fn:true`, `fn:false`, `fn:not`, `fn:boolean`

#### Numeric (high priority)
`fn:number`, `fn:abs`, `fn:ceiling`, `fn:floor`, `fn:round`, `fn:round-half-to-even`, `fn:format-integer`, `fn:format-number`

#### String (high priority — commonly used)
`fn:concat`, `fn:string-join`, `fn:substring`, `fn:string-length`, `fn:normalize-space`, `fn:normalize-unicode`, `fn:upper-case`, `fn:lower-case`, `fn:translate`, `fn:starts-with`, `fn:ends-with`, `fn:substring-before`, `fn:substring-after`, `fn:matches`, `fn:replace`, `fn:tokenize`, `fn:compare`, `fn:codepoint-equal`, `fn:codepoints-to-string`, `fn:string-to-codepoints`, `fn:analyze-string`, `fn:encode-for-uri`, `fn:iri-to-uri`, `fn:escape-html-uri`

#### Sequence (high priority — commonly used)
`fn:empty`, `fn:exists`, `fn:count`, `fn:head`, `fn:tail`, `fn:insert-before`, `fn:remove`, `fn:reverse`, `fn:subsequence`, `fn:unordered`, `fn:distinct-values`, `fn:index-of`, `fn:deep-equal`, `fn:zero-or-one`, `fn:one-or-more`, `fn:exactly-one`, `fn:avg`, `fn:max`, `fn:min`, `fn:sum`

#### Node (medium priority)
`fn:name`, `fn:local-name`, `fn:namespace-uri`, `fn:lang`, `fn:path`, `fn:has-children`, `fn:innermost`, `fn:outermost`

#### Accessor (medium priority)
`fn:node-name`, `fn:nilled`, `fn:base-uri`, `fn:document-uri`

#### Context (medium priority)
`fn:position`, `fn:last` (if not handled already via predicate context)

#### Higher-order (medium priority)
`fn:for-each`, `fn:filter`, `fn:fold-left`, `fn:fold-right`, `fn:for-each-pair`, `fn:sort`, `fn:apply`, `fn:function-lookup`, `fn:function-name`, `fn:function-arity`

#### Map functions (medium priority)
`map:merge`, `map:size`, `map:keys`, `map:contains`, `map:get`, `map:find`, `map:put`, `map:entry`, `map:remove`, `map:for-each`

#### Array functions (medium priority)
`array:size`, `array:get`, `array:put`, `array:append`, `array:subarray`, `array:remove`, `array:insert-before`, `array:head`, `array:tail`, `array:reverse`, `array:join`, `array:for-each`, `array:filter`, `array:fold-left`, `array:fold-right`, `array:for-each-pair`, `array:sort`, `array:flatten`

#### Math functions (low priority)
`math:pi`, `math:exp`, `math:exp10`, `math:log`, `math:log10`, `math:pow`, `math:sqrt`, `math:sin`, `math:cos`, `math:tan`, `math:asin`, `math:acos`, `math:atan`, `math:atan2`

#### Error/Trace (low priority)
`fn:error`, `fn:trace`

#### QName functions (low priority — less relevant for HTML)
`fn:QName`, `fn:prefix-from-QName`, `fn:local-name-from-QName`, `fn:namespace-uri-from-QName`, `fn:namespace-uri-for-prefix`, `fn:in-scope-prefixes`, `fn:resolve-QName`

#### Date/Time functions (low priority — require atomic type support first)
All `*-from-duration`, `*-from-dateTime`, `*-from-date`, `*-from-time`, `fn:current-dateTime`, `fn:current-date`, `fn:current-time`, `fn:format-dateTime`, `fn:format-date`, `fn:format-time`, `fn:adjust-*-to-timezone`, `fn:implicit-timezone`

#### Document functions (low priority — less relevant for in-memory HTML)
`fn:doc`, `fn:doc-available`, `fn:collection`, `fn:uri-collection`, `fn:unparsed-text`, `fn:unparsed-text-lines`, `fn:unparsed-text-available`, `fn:environment-variable`, `fn:available-environment-variables`

#### Parsing/Serialization (low priority)
`fn:parse-xml`, `fn:parse-xml-fragment`, `fn:serialize`, `fn:json-doc`, `fn:parse-json`, `fn:json-to-xml`, `fn:xml-to-json`

#### ID functions (low priority)
`fn:id`, `fn:idref`, `fn:element-with-id`, `fn:generate-id`

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
| AfterBody — Comment token | `todo!()` — should insert as last child of `<html>` | `src/html/grammar/insertion_mode_impls/mod.rs:700` |
| AfterBody — DocType token | `todo!()` — should be a parse error (ignore) | `src/html/grammar/insertion_mode_impls/mod.rs:703` |
| AfterAfterBody — Comment token | `todo!()` — should insert as last child of Document | `src/html/grammar/insertion_mode_impls/mod.rs:739` |
| Declarative Shadow DOM | Not supported (noted in comment) | `src/html/grammar/insertion_mode_impls/mod.rs:317` |
| HTML fragment parsing algorithm | Not implemented (noted in TODO comment) | `src/html/grammar/insertion_mode_impls/mod.rs:712` |

### Not Implemented

| Feature | Notes |
|---------|-------|
| `<math>` and `<svg>` foreign content | WHATWG defines special parsing rules for MathML and SVG embedded in HTML |
| Adoption agency algorithm edge cases | Implementation exists but full spec coverage not verified |
| Foster parenting | May have edge cases not covered |

---

## Data Model Gaps

| Feature | Gap | Location |
|---------|-----|----------|
| `PINode` (Processing Instruction) | Struct is empty — no `target` or `data` fields | `src/xpath/grammar/data_model/mod.rs:723` |
| Namespace nodes | Not represented in the tree at all | N/A |
| `is_root_level` naming | Misleadingly named; should be `is_initial_step` per TODO | `src/xpath/mod.rs:236` |

---

## Error Handling Gaps

| Feature | Gap | Location |
|---------|-----|----------|
| `fn:data()` on function items | Should raise `err:FOTY0013`, returns placeholder instead | `static_function_calls.rs:275` |
| `fn:string()` on function items | Should raise error per spec, returns placeholder instead | `static_function_calls.rs:306` |
| Function dispatch | Unknown functions return a generic error; spec defines `err:XPST0017` | `static_function_calls.rs` |
