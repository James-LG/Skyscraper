# Features Backlog

Missing features noted by `todo!()` stubs in the codebase. Organized by module.

---

## HTML Parser — Insertion Modes

**File:** `src/html/grammar/mod.rs`

| Line(s) | Feature | WHATWG Section |
|---------|---------|----------------|
| ~~528~~ | ~~Foster parenting — adjusted insertion location logic~~ | ~~[13.2.6.1](https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node)~~ **DONE** |
| ~~1334~~ | ~~InHeadNoscript insertion mode~~ | ~~[13.2.6.4.5](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inheadnoscript)~~ **DONE** |
| ~~1338~~ | ~~InTable insertion mode~~ | ~~[13.2.6.4.9](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intable)~~ **DONE** |
| ~~1339~~ | ~~InTableText insertion mode~~ | ~~[13.2.6.4.10](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intabletext)~~ **DONE** |
| ~~1340~~ | ~~InCaption insertion mode~~ | ~~[13.2.6.4.11](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incaption)~~ **DONE** |
| ~~1341~~ | ~~InColumnGroup insertion mode~~ | ~~[13.2.6.4.12](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incolumngroup)~~ **DONE** |
| ~~1342~~ | ~~InTableBody insertion mode~~ | ~~[13.2.6.4.13](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-intablebody)~~ **DONE** |
| ~~1343~~ | ~~InRow insertion mode~~ | ~~[13.2.6.4.14](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inrow)~~ **DONE** |
| ~~1344~~ | ~~InCell insertion mode~~ | ~~[13.2.6.4.15](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incell)~~ **DONE** |
| ~~1345~~ | ~~InSelect insertion mode~~ | ~~[13.2.6.4.16](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inselect)~~ **DONE** |
| ~~1346~~ | ~~InSelectInTable insertion mode~~ | ~~[13.2.6.4.17](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inselectintable)~~ **DONE** |
| ~~1349~~ | ~~InFrameset insertion mode~~ | ~~[13.2.6.4.19](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inframeset)~~ **DONE** |
| ~~1350~~ | ~~AfterFrameset insertion mode~~ | ~~[13.2.6.4.20](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterframeset)~~ **DONE** |
| ~~1352~~ | ~~AfterAfterFrameset insertion mode~~ | ~~[13.2.6.4.22](https://html.spec.whatwg.org/multipage/parsing.html#the-after-after-frameset-insertion-mode)~~ **DONE** |

## HTML Parser — BeforeHtml Insertion Mode

**File:** `src/html/grammar/insertion_mode_impls/mod.rs`

| Line | Feature |
|------|---------|
| ~~98~~ | ~~DOCTYPE token handling~~ **DONE** |
| ~~138~~ | ~~Unexpected end tags (parse error)~~ **DONE** |

## HTML Parser — BeforeHead Insertion Mode

**File:** `src/html/grammar/insertion_mode_impls/mod.rs`

| Line | Feature |
|------|---------|
| ~~178~~ | ~~DOCTYPE token handling~~ **DONE** |
| ~~180~~ | ~~`<html>` start tag (process using InBody rules)~~ **DONE** |
| ~~195~~ | ~~Unexpected end tags (parse error)~~ **DONE** |

## HTML Parser — InHead Insertion Mode

**File:** `src/html/grammar/insertion_mode_impls/mod.rs`

| Line | Feature |
|------|---------|
| ~~233~~ | ~~DOCTYPE token handling~~ **DONE** |
| ~~235~~ | ~~`<html>` start tag (process using InBody rules)~~ **DONE** |
| ~~266~~ | ~~`<noscript>` start tag~~ **DONE** |
| ~~305~~ | ~~`<template>` start tag — non-adjusted-current-node branch~~ **DONE** |
| ~~330~~ | ~~Duplicate `<head>` start tag (parse error)~~ **DONE** |
| ~~333~~ | ~~Unexpected end tags (parse error)~~ **DONE** |

## HTML Parser — AfterHead Insertion Mode

**File:** `src/html/grammar/insertion_mode_impls/mod.rs`

| Line | Feature |
|------|---------|
| ~~370~~ | ~~Comment token handling~~ **DONE** |
| ~~371~~ | ~~DOCTYPE token handling~~ **DONE** |
| ~~373~~ | ~~`<html>` start tag (process using InBody rules)~~ **DONE** |
| ~~383~~ | ~~`<frameset>` start tag~~ **DONE** |
| ~~392~~ | ~~Head-level elements after head (`<base>`, `<link>`, `<meta>`, `<script>`, `<style>`, `<template>`, `<title>`, etc.)~~ **DONE** |
| ~~395~~ | ~~`</template>` end tag~~ **DONE** |
| ~~403~~ | ~~Duplicate `<head>` start tag (parse error)~~ **DONE** |
| ~~406~~ | ~~Unexpected end tags (parse error)~~ **DONE** |

## HTML Parser — InBody Insertion Mode

**File:** `src/html/grammar/insertion_mode_impls/in_body_insertion_mode.rs`

| Line(s) | Feature |
|---------|---------|
| ~~47~~ | ~~NULL character handling~~ **DONE** |
| ~~74~~ | ~~DOCTYPE token handling~~ **DONE** |
| ~~153~~ | ~~`<frameset>` start tag~~ **DONE** |
| 244 | `<pre>` / `<listing>` start tags |
| 348 | `<dd>` / `<dt>` start tags |
| 351 | `<plaintext>` start tag |
| 507 | `</dd>` / `</dt>` end tags |
| 533 | `</sarcasm>` end tag (spec joke — "take a deep breath") |
| 578 | `<nobr>` start tag |
| 592 | `<applet>` / `<marquee>` / `<object>` start tags |
| 597 | `</applet>` / `</marquee>` / `</object>` end tags |
| ~~600~~ | ~~`<table>` start tag~~ **DONE** |
| 608 | Void elements: `<area>`, `<br>`, `<embed>`, `<img>` etc. — some missing branches |
| 658 | `<param>` / `<source>` / `<track>` start tags |
| 661 | `<hr>` start tag |
| 664 | `<image>` start tag (should be rewritten to `<img>`) |
| 681 | `<xmp>` start tag |
| 684 | `<iframe>` start tag |
| 689 | `<noembed>` / `<noscript>` start tags |
| ~~692~~ | ~~`<select>` start tag~~ **DONE** |
| ~~697~~ | ~~`<optgroup>` / `<option>` start tags~~ **DONE** |
| 702 | `<rb>` / `<rtc>` start tags (ruby) |
| 707 | `<rp>` / `<rt>` start tags (ruby) |
| 710 | `<math>` start tag (MathML integration) |
| 733 | Table-related start tags in body (`<caption>`, `<col>`, `<colgroup>`, `<frame>`, `<head>`, `<tbody>`, `<td>`, `<tfoot>`, `<th>`, `<thead>`, `<tr>`) |

---

## XPath — Expressions

### For / Let / Quantified / If Expressions

**File:** `src/xpath/grammar/expressions/mod.rs`

| Line | Feature |
|------|---------|
| 387 | `for` expressions (`for $x in ... return ...`) |
| 388 | `let` expressions (`let $x := ... return ...`) |
| 389 | Quantified expressions (`some`/`every ... satisfies ...`) |
| 390 | `if` expressions (`if (...) then ... else ...`) |

### Logical Expressions

**File:** `src/xpath/grammar/expressions/logical_expressions.rs`

| Line | Feature |
|------|---------|
| 60 | `or` operator |
| 108 | `and` operator |

### Comparison Expressions

**File:** `src/xpath/grammar/expressions/comparison_expressions.rs`

| Line | Feature |
|------|---------|
| 124 | Value comparisons (`eq`, `ne`, `lt`, `le`, `gt`, `ge`) |
| 126 | Node comparisons (`is`, `<<`, `>>`) |

### Arithmetic Expressions

**File:** `src/xpath/grammar/expressions/arithmetic_expressions.rs`

| Line | Feature |
|------|---------|
| 87 | Additive operators (`+`, `-`) |
| 186 | Multiplicative operators (`*`, `div`, `idiv`, `mod`) |
| 272 | Unary operators (`+`, `-` prefix) |

### String Concat Expression

**File:** `src/xpath/grammar/expressions/string_concat_expressions.rs`

| Line | Feature |
|------|---------|
| 59 | String concatenation operator (`||`) |

### Arrow Operator

**File:** `src/xpath/grammar/expressions/arrow_operator.rs`

| Line | Feature |
|------|---------|
| 86 | Arrow operator (`=>`) |

### Simple Map Operator

**File:** `src/xpath/grammar/expressions/simple_map_operator.rs`

| Line | Feature |
|------|---------|
| 60 | Simple map operator (`!`) |

### Sequence Expressions

**File:** `src/xpath/grammar/expressions/sequence_expressions/`

| Line | File | Feature |
|------|------|---------|
| 66 | `constructing_sequences.rs` | Range operator (`to`) |
| 84 | `combining_node_sequences.rs` | Union operator (`union` / `\|`) |
| 175 | `combining_node_sequences.rs` | Intersect/except operators |

### Expressions on Sequence Types

**File:** `src/xpath/grammar/expressions/expressions_on_sequence_types/`

| Line | File | Feature |
|------|------|---------|
| 72 | `instance_of.rs` | `instance of` operator |
| 65 | `cast.rs` | `cast as` operator |
| 68 | `castable.rs` | `castable as` operator |

### Postfix Expressions

**File:** `src/xpath/grammar/expressions/postfix_expressions.rs`

| Line | Feature |
|------|---------|
| 84 | Postfix expression evaluation (predicate/argument chains on primary expressions) |

### Primary Expressions

**File:** `src/xpath/grammar/expressions/primary_expressions/mod.rs`

| Line | Feature |
|------|---------|
| 147 | Variable references (`$var`) |
| 154 | Function item expressions (named function references, inline functions) |
| 155 | Map constructors (`map { ... }`) |
| 156 | Array constructors (`[ ... ]` / `array { ... }`) |
| 157 | Unary lookup (`?key`) |

### Static Function Calls

**File:** `src/xpath/grammar/expressions/primary_expressions/static_function_calls.rs`

| Line | Feature |
|------|---------|
| 76 | URI-qualified function names |
| 147–148 | `fn:data()` for PI and comment nodes |
| 155 | `fn:data()` for function items |
| 169–170 | `fn:string()` for PI and comment nodes |
| 182 | `fn:string()` for function items |

## XPath — Path Expression Axes

### Forward Axes

**File:** `src/xpath/grammar/expressions/path_expressions/steps/forward_step.rs`

| Line | Feature |
|------|---------|
| 90 | `self::` axis |
| 92 | `following-sibling::` axis |
| 93 | `following::` axis |
| 94 | `namespace::` axis |

### Reverse Axes

**File:** `src/xpath/grammar/expressions/path_expressions/steps/reverse_step.rs`

| Line | Feature |
|------|---------|
| 85 | `ancestor::` axis |
| 86 | `preceding-sibling::` axis |
| 87 | `preceding::` axis |
| 88 | `ancestor-or-self::` axis |

### Node Tests

**File:** `src/xpath/grammar/expressions/path_expressions/steps/node_tests.rs`

| Line | Feature |
|------|---------|
| 120 | NameTest evaluation for non-node items |
| 139 | Prefixed name matching (`prefix:local`) |
| 142 | URI-qualified name matching |
| 239 | Wildcard — prefixed name (`prefix:*`) |
| 240 | Wildcard — suffixed name (`*:local`) |
| 241 | Wildcard — braced URI (`Q{uri}*`) |

## XPath — Type System

### Kind Tests

**File:** `src/xpath/grammar/types/mod.rs`

| Line | Feature |
|------|---------|
| 173 | `comment()` kind test |
| 174 | `namespace-node()` kind test |
| 176 | `element()` kind test (with parameters) |
| 190 | `schema-element()` kind test |
| 191 | `schema-attribute()` kind test |
| 192 | `processing-instruction()` kind test |
| 277 | `document-node()` kind test with element argument |
| 317 | `schema-attribute()` display formatting |
| 362 | `processing-instruction()` display formatting |

### Sequence Types

**File:** `src/xpath/grammar/types/sequence_type.rs`

| Line | Feature |
|------|---------|
| 82 | Occurrence indicator matching (`?`, `*`, `+`) |
| 194 | `function()` type test |
| 195 | `map()` type test |
| 196 | `array()` type test |
| 197 | Atomic/union type test |

### Attribute Tests

**File:** `src/xpath/grammar/types/attribute_test.rs`

| Line | Feature |
|------|---------|
| 103 | Attribute test with type name parameter |
| 150 | Attribute name matching |

## XPath — Data Model

**File:** `src/xpath/grammar/data_model/mod.rs`

| Line | Feature |
|------|---------|
| 92 | Function node display formatting |
| 633 | Processing instruction node display formatting |

## XPath — Maps and Arrays

**File:** `src/xpath/grammar/expressions/maps_and_arrays/arrays.rs`

| Line | Feature |
|------|---------|
| 47 | Array constructor display formatting |

## XPath — Common Helpers

**File:** `src/xpath/grammar/expressions/common.rs`

| Line | Feature |
|------|---------|
| 90 | Argument placeholder evaluation (`?`) |
