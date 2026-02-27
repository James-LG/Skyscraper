//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-context-item-expression>

use std::fmt::Display;

use nom::error::context;

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, Function, XpathItem},
            expressions::common::{argument_list, Argument, ArgumentList},
            recipes::Res,
            types::{eq_name, EQName},
            whitespace_recipes::ws,
            xml_names::QName,
            XpathItemTreeNode,
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XpathExpressionContext, XpathItemTree,
    },
    xpath_item_set,
};

pub fn function_call(input: &str) -> Res<&str, FunctionCall> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-FunctionCall

    context("function_call", ws((eq_name, argument_list)))(input).map(|(next_input, res)| {
        (
            next_input,
            FunctionCall {
                name: res.0,
                argument_list: res.1,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionCall {
    pub name: EQName,
    pub argument_list: ArgumentList,
}

impl Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.name, self.argument_list)
    }
}

impl FunctionCall {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // fn:root is special: its arguments are never evaluated because path
        // expansion passes `self::node()` which may hit unimplemented axes.
        let is_fn_root = match &self.name {
            EQName::QName(QName::PrefixedName(p)) => {
                p.prefix == "fn" && p.local_part == "root"
            }
            EQName::UriQualifiedName(uqn) => {
                uqn.uri == XPATH_FUNCTIONS_NS && uqn.name == "root"
            }
            _ => false,
        };
        if is_fn_root {
            return Ok(xpath_item_set![XpathItem::Node(context.item_tree.root())]);
        }

        // Check for partial function application (argument placeholders).
        let has_placeholder = self
            .argument_list
            .0
            .iter()
            .any(|a| matches!(a, Argument::ArgumentPlaceHolder));

        if has_placeholder {
            // Build an inline function that captures the concrete arguments
            // and leaves placeholders as parameters.
            let mut params = Vec::new();
            let mut body_args = Vec::new();
            let mut placeholder_idx = 0u32;

            for arg in &self.argument_list.0 {
                match arg {
                    Argument::ArgumentPlaceHolder => {
                        let param_name = format!("__placeholder_{}", placeholder_idx);
                        body_args.push(format!("${}", param_name));
                        params.push(param_name);
                        placeholder_idx += 1;
                    }
                    Argument::ExprSingle(expr) => {
                        body_args.push(format!("{}", expr));
                    }
                }
            }

            let body_source = format!("{}({})", self.name, body_args.join(", "));
            return Ok(xpath_item_set![XpathItem::Function(Function::Inline {
                params,
                body_source,
            })]);
        }

        // Eagerly evaluate arguments, then dispatch through the shared table.
        let mut args = Vec::new();
        for arg in &self.argument_list.0 {
            args.push(arg.eval(context)?);
        }
        dispatch_function(&self.name, &args, context)
    }
}

/// Dispatch a function call by name with pre-evaluated arguments.
///
/// Used by `ArrowExpr::eval` where the left-hand side is prepended as the
/// first argument.

/// The XPath Functions namespace URI.
const XPATH_FUNCTIONS_NS: &str = "http://www.w3.org/2005/xpath-functions";

pub(crate) fn dispatch_function<'tree>(
    name: &EQName,
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    // Extract the local name for function dispatch. The fn: prefix and the
    // XPath Functions namespace URI both resolve to the same set of functions.
    // Unprefixed names are also tried against the default function namespace.
    let local_name = match name {
        EQName::QName(qname) => match qname {
            QName::PrefixedName(p) if p.prefix == "fn" => Some(p.local_part.as_str()),
            QName::PrefixedName(_) => None,
            QName::UnprefixedName(n) => Some(n.as_str()),
        },
        EQName::UriQualifiedName(uqn) if uqn.uri == XPATH_FUNCTIONS_NS => {
            Some(uqn.name.as_str())
        }
        EQName::UriQualifiedName(_) => None,
    };

    if let Some(local) = local_name {
        if let Some(result) = dispatch_by_local_name(local, args, context)? {
            return Ok(result);
        }
    }

    Err(ExpressionApplyError {
        msg: format!("Unknown function {}", name),
    })
}

/// Dispatch a function by its local name. Returns `Ok(Some(result))` if the
/// function is known, `Ok(None)` if unrecognized, or `Err` on evaluation error.
fn dispatch_by_local_name<'tree>(
    local_name: &str,
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
) -> Result<Option<XpathItemSet<'tree>>, ExpressionApplyError> {
    match local_name {
        "root" => Ok(Some(
            xpath_item_set![XpathItem::Node(context.item_tree.root())],
        )),
        "contains" => func_contains(args, context).map(Some),
        "data" => {
            let target = if args.is_empty() {
                xpath_item_set![context.item.clone()]
            } else {
                args[0].clone()
            };
            let atoms = func_data(&target, context.item_tree);
            Ok(Some(
                atoms
                    .into_iter()
                    .map(|a| XpathItem::AnyAtomicType(a))
                    .collect(),
            ))
        }
        "string" => {
            let target = if args.is_empty() {
                &context.item
            } else if args[0].len() == 1 {
                &args[0][0]
            } else {
                return Err(ExpressionApplyError::new(format!(
                    "fn:string expects 0 or 1 argument, got sequence of length {}",
                    args[0].len()
                )));
            };
            let s = func_string(target, context.item_tree);
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(s)
            )]))
        }
        // Boolean functions
        // https://www.w3.org/TR/xpath-functions-31/#func-true
        "true" => {
            check_arity("fn:true", args, 0)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(true)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-false
        "false" => {
            check_arity("fn:false", args, 0)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(false)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-not
        "not" => {
            check_arity("fn:not", args, 1)?;
            let ebv = args[0].boolean();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(!ebv)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-boolean
        "boolean" => {
            check_arity("fn:boolean", args, 1)?;
            let ebv = args[0].boolean();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(ebv)
            )]))
        }
        // Numeric functions
        // https://www.w3.org/TR/xpath-functions-31/#func-number
        "number" => {
            let target = if args.is_empty() {
                &context.item
            } else {
                check_arity("fn:number", args, 1)?;
                if args[0].len() != 1 {
                    return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::Double(ordered_float::OrderedFloat(f64::NAN))
                    )]));
                }
                &args[0][0]
            };
            let s = func_string(target, context.item_tree);
            let val = s.trim().parse::<f64>().unwrap_or(f64::NAN);
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Double(ordered_float::OrderedFloat(val))
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-abs
        "abs" => {
            check_arity("fn:abs", args, 1)?;
            func_numeric_unary(&args[0], |i| i.abs(), |f| f.abs(), |d| d.abs()).map(Some)
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-ceiling
        "ceiling" => {
            check_arity("fn:ceiling", args, 1)?;
            func_numeric_unary(&args[0], |i| i, |f| f.ceil(), |d| d.ceil()).map(Some)
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-floor
        "floor" => {
            check_arity("fn:floor", args, 1)?;
            func_numeric_unary(&args[0], |i| i, |f| f.floor(), |d| d.floor()).map(Some)
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-round
        "round" => {
            check_arity("fn:round", args, 1)?;
            func_numeric_unary(&args[0], |i| i, |f| f.round(), |d| d.round()).map(Some)
        }
        // String functions
        // https://www.w3.org/TR/xpath-functions-31/#func-concat
        "concat" => {
            if args.len() < 2 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:concat requires at least 2 arguments, got {}",
                    args.len()
                )));
            }
            let mut result = String::new();
            for arg in args {
                if arg.is_empty() {
                    // empty sequence → empty string
                } else {
                    result.push_str(&func_string(&arg[0], context.item_tree));
                }
            }
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-string-join
        "string-join" => {
            if args.is_empty() || args.len() > 2 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:string-join expects 1 or 2 arguments, got {}",
                    args.len()
                )));
            }
            let separator = if args.len() == 2 {
                if args[1].is_empty() {
                    String::new()
                } else {
                    func_string(&args[1][0], context.item_tree)
                }
            } else {
                String::new()
            };
            let parts: Vec<String> = args[0]
                .iter()
                .map(|item| func_string(item, context.item_tree))
                .collect();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(parts.join(&separator))
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-string-length
        "string-length" => {
            let s = if args.is_empty() {
                func_string(&context.item, context.item_tree)
            } else {
                check_arity("fn:string-length", args, 1)?;
                if args[0].is_empty() {
                    String::new()
                } else {
                    func_string(&args[0][0], context.item_tree)
                }
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Integer(s.chars().count() as i64)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-normalize-space
        "normalize-space" => {
            let s = if args.is_empty() {
                func_string(&context.item, context.item_tree)
            } else {
                check_arity("fn:normalize-space", args, 1)?;
                if args[0].is_empty() {
                    String::new()
                } else {
                    func_string(&args[0][0], context.item_tree)
                }
            };
            let normalized = s.split_whitespace().collect::<Vec<_>>().join(" ");
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(normalized)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-upper-case
        "upper-case" => {
            check_arity("fn:upper-case", args, 1)?;
            let s = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(s.to_uppercase())
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-lower-case
        "lower-case" => {
            check_arity("fn:lower-case", args, 1)?;
            let s = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(s.to_lowercase())
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-starts-with
        "starts-with" => {
            check_arity("fn:starts-with", args, 2)?;
            let haystack = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            let needle = if args[1].is_empty() {
                String::new()
            } else {
                func_string(&args[1][0], context.item_tree)
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(haystack.starts_with(&needle))
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-ends-with
        "ends-with" => {
            check_arity("fn:ends-with", args, 2)?;
            let haystack = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            let needle = if args[1].is_empty() {
                String::new()
            } else {
                func_string(&args[1][0], context.item_tree)
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(haystack.ends_with(&needle))
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-substring
        "substring" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:substring expects 2 or 3 arguments, got {}",
                    args.len()
                )));
            }
            let s = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            let chars: Vec<char> = s.chars().collect();
            // XPath uses 1-based indexing with rounding.
            let start_double = extract_double(&args[1], context.item_tree)?;
            let start = (start_double.round() as i64 - 1).max(0) as usize;
            let result = if args.len() == 3 {
                let len_double = extract_double(&args[2], context.item_tree)?;
                let end = ((start_double.round() + len_double.round()) as i64 - 1).max(0) as usize;
                let end = end.min(chars.len());
                let start = start.min(chars.len());
                chars[start..end].iter().collect()
            } else {
                let start = start.min(chars.len());
                chars[start..].iter().collect()
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-substring-before
        "substring-before" => {
            check_arity("fn:substring-before", args, 2)?;
            let s = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            let sub = if args[1].is_empty() {
                String::new()
            } else {
                func_string(&args[1][0], context.item_tree)
            };
            let result = s.find(&sub).map(|i| &s[..i]).unwrap_or("").to_string();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-substring-after
        "substring-after" => {
            check_arity("fn:substring-after", args, 2)?;
            let s = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            let sub = if args[1].is_empty() {
                String::new()
            } else {
                func_string(&args[1][0], context.item_tree)
            };
            let result = s
                .find(&sub)
                .map(|i| &s[i + sub.len()..])
                .unwrap_or("")
                .to_string();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-translate
        "translate" => {
            check_arity("fn:translate", args, 3)?;
            let s = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            let map_from: Vec<char> = if args[1].is_empty() {
                Vec::new()
            } else {
                func_string(&args[1][0], context.item_tree).chars().collect()
            };
            let map_to: Vec<char> = if args[2].is_empty() {
                Vec::new()
            } else {
                func_string(&args[2][0], context.item_tree).chars().collect()
            };
            let result: String = s
                .chars()
                .filter_map(|c| {
                    if let Some(pos) = map_from.iter().position(|&fc| fc == c) {
                        map_to.get(pos).copied() // None if pos >= map_to.len() → remove char
                    } else {
                        Some(c)
                    }
                })
                .collect();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // Sequence functions
        // https://www.w3.org/TR/xpath-functions-31/#func-empty
        "empty" => {
            check_arity("fn:empty", args, 1)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(args[0].is_empty())
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-exists
        "exists" => {
            check_arity("fn:exists", args, 1)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(!args[0].is_empty())
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-count
        "count" => {
            check_arity("fn:count", args, 1)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Integer(args[0].len() as i64)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-head
        "head" => {
            check_arity("fn:head", args, 1)?;
            if args[0].is_empty() {
                Ok(Some(XpathItemSet::new()))
            } else {
                Ok(Some(xpath_item_set![args[0][0].clone()]))
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-tail
        "tail" => {
            check_arity("fn:tail", args, 1)?;
            if args[0].len() <= 1 {
                Ok(Some(XpathItemSet::new()))
            } else {
                Ok(Some(args[0].iter().skip(1).cloned().collect()))
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-reverse
        "reverse" => {
            check_arity("fn:reverse", args, 1)?;
            let reversed: Vec<_> = args[0].iter().rev().cloned().collect();
            Ok(Some(reversed.into_iter().collect()))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-distinct-values
        "distinct-values" => {
            check_arity("fn:distinct-values", args, 1)?;
            // XpathItemSet is already an IndexSet, so duplicates are removed.
            // However, we need to atomize values first for proper comparison.
            let atoms = func_data(&args[0], context.item_tree);
            Ok(Some(
                atoms
                    .into_iter()
                    .map(|a| XpathItem::AnyAtomicType(a))
                    .collect(),
            ))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-sum
        "sum" => func_sum(args, context).map(Some),
        // Node functions
        // https://www.w3.org/TR/xpath-functions-31/#func-name
        "name" => {
            let target = if args.is_empty() {
                &context.item
            } else {
                check_arity("fn:name", args, 1)?;
                if args[0].is_empty() {
                    return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::String(String::new())
                    )]));
                }
                &args[0][0]
            };
            let name = match target {
                XpathItem::Node(node) => match node {
                    XpathItemTreeNode::ElementNode(e) => e.name.clone(),
                    XpathItemTreeNode::AttributeNode(a) => a.name.clone(),
                    _ => String::new(),
                },
                _ => String::new(),
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(name)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-local-name
        // In this HTML-only processor, local-name is identical to name
        // because elements are not namespace-qualified.
        "local-name" => {
            let target = if args.is_empty() {
                &context.item
            } else {
                check_arity("fn:local-name", args, 1)?;
                if args[0].is_empty() {
                    return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::String(String::new())
                    )]));
                }
                &args[0][0]
            };
            let name = match target {
                XpathItem::Node(node) => match node {
                    XpathItemTreeNode::ElementNode(e) => e.name.clone(),
                    XpathItemTreeNode::AttributeNode(a) => a.name.clone(),
                    _ => String::new(),
                },
                _ => String::new(),
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(name)
            )]))
        }
        // Context functions
        // https://www.w3.org/TR/xpath-functions-31/#func-position
        "position" => {
            check_arity("fn:position", args, 0)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Integer(context.position as i64)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-last
        "last" => {
            check_arity("fn:last", args, 0)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Integer(context.size as i64)
            )]))
        }
        // String functions (regex-based)
        // https://www.w3.org/TR/xpath-functions-31/#func-matches
        "matches" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:matches expects 2 or 3 arguments, got {}",
                    args.len()
                )));
            }
            let input = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            let pattern = if args[1].is_empty() {
                String::new()
            } else {
                func_string(&args[1][0], context.item_tree)
            };
            let flags = if args.len() == 3 && !args[2].is_empty() {
                func_string(&args[2][0], context.item_tree)
            } else {
                String::new()
            };
            let re = build_regex(&pattern, &flags)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(re.is_match(&input))
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-replace
        "replace" => {
            if args.len() < 3 || args.len() > 4 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:replace expects 3 or 4 arguments, got {}",
                    args.len()
                )));
            }
            let input = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            let pattern = if args[1].is_empty() {
                String::new()
            } else {
                func_string(&args[1][0], context.item_tree)
            };
            let replacement = if args[2].is_empty() {
                String::new()
            } else {
                func_string(&args[2][0], context.item_tree)
            };
            let flags = if args.len() == 4 && !args[3].is_empty() {
                func_string(&args[3][0], context.item_tree)
            } else {
                String::new()
            };
            let re = build_regex(&pattern, &flags)?;
            let result = re.replace_all(&input, replacement.as_str()).to_string();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-tokenize
        "tokenize" => {
            if args.is_empty() || args.len() > 3 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:tokenize expects 1-3 arguments, got {}",
                    args.len()
                )));
            }
            let input = if args[0].is_empty() {
                String::new()
            } else {
                func_string(&args[0][0], context.item_tree)
            };
            if args.len() == 1 {
                // 1-arg form: normalize whitespace and split on whitespace.
                let normalized = input.trim();
                if normalized.is_empty() {
                    return Ok(Some(XpathItemSet::new()));
                }
                let tokens: XpathItemSet = normalized
                    .split_whitespace()
                    .map(|s| XpathItem::AnyAtomicType(AnyAtomicType::String(s.to_string())))
                    .collect();
                return Ok(Some(tokens));
            }
            let pattern = if args[1].is_empty() {
                String::new()
            } else {
                func_string(&args[1][0], context.item_tree)
            };
            let flags = if args.len() == 3 && !args[2].is_empty() {
                func_string(&args[2][0], context.item_tree)
            } else {
                String::new()
            };
            let re = build_regex(&pattern, &flags)?;
            let tokens: XpathItemSet = re
                .split(&input)
                .filter(|s| !s.is_empty())
                .map(|s| XpathItem::AnyAtomicType(AnyAtomicType::String(s.to_string())))
                .collect();
            Ok(Some(tokens))
        }
        // Remaining sequence functions
        // https://www.w3.org/TR/xpath-functions-31/#func-subsequence
        "subsequence" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:subsequence expects 2 or 3 arguments, got {}",
                    args.len()
                )));
            }
            let start_double = extract_double(&args[1], context.item_tree)?;
            let start = (start_double.round() as i64 - 1).max(0) as usize;
            let seq_len = args[0].len();
            let start = start.min(seq_len);
            if args.len() == 3 {
                let len_double = extract_double(&args[2], context.item_tree)?;
                let end = ((start_double.round() + len_double.round()) as i64 - 1).max(0) as usize;
                let end = end.min(seq_len);
                Ok(Some(args[0].iter().skip(start).take(end - start).cloned().collect()))
            } else {
                Ok(Some(args[0].iter().skip(start).cloned().collect()))
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-insert-before
        "insert-before" => {
            check_arity("fn:insert-before", args, 3)?;
            let pos_double = extract_double(&args[1], context.item_tree)?;
            let pos = (pos_double.round() as i64 - 1).max(0) as usize;
            let pos = pos.min(args[0].len());
            let mut result: Vec<XpathItem> = args[0].iter().cloned().collect();
            for (i, item) in args[2].iter().enumerate() {
                result.insert(pos + i, item.clone());
            }
            Ok(Some(result.into_iter().collect()))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-remove
        "remove" => {
            check_arity("fn:remove", args, 2)?;
            let pos_double = extract_double(&args[1], context.item_tree)?;
            let pos = pos_double.round() as i64;
            // 1-based position; if out of range, return sequence unchanged.
            if pos < 1 || pos as usize > args[0].len() {
                return Ok(Some(args[0].clone()));
            }
            let idx = (pos - 1) as usize;
            Ok(Some(
                args[0]
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != idx)
                    .map(|(_, item)| item.clone())
                    .collect(),
            ))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-index-of
        "index-of" => {
            check_arity("fn:index-of", args, 2)?;
            if args[1].len() != 1 {
                return Err(ExpressionApplyError::new(
                    "fn:index-of: search value must be a single item".to_string(),
                ));
            }
            let search = &args[1][0];
            let positions: XpathItemSet = args[0]
                .iter()
                .enumerate()
                .filter(|(_, item)| *item == search)
                .map(|(i, _)| XpathItem::AnyAtomicType(AnyAtomicType::Integer(i as i64 + 1)))
                .collect();
            Ok(Some(positions))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-zero-or-one
        "zero-or-one" => {
            check_arity("fn:zero-or-one", args, 1)?;
            if args[0].len() > 1 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:zero-or-one: sequence has {} items",
                    args[0].len()
                )));
            }
            Ok(Some(args[0].clone()))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-one-or-more
        "one-or-more" => {
            check_arity("fn:one-or-more", args, 1)?;
            if args[0].is_empty() {
                return Err(ExpressionApplyError::new(
                    "fn:one-or-more: sequence is empty".to_string(),
                ));
            }
            Ok(Some(args[0].clone()))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-exactly-one
        "exactly-one" => {
            check_arity("fn:exactly-one", args, 1)?;
            if args[0].len() != 1 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:exactly-one: sequence has {} items",
                    args[0].len()
                )));
            }
            Ok(Some(args[0].clone()))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-avg
        "avg" => {
            check_arity("fn:avg", args, 1)?;
            if args[0].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            let atoms = func_data(&args[0], context.item_tree);
            let mut total: f64 = 0.0;
            for atom in &atoms {
                match atom {
                    AnyAtomicType::Integer(n) => total += *n as f64,
                    AnyAtomicType::Float(f) => total += f.0 as f64,
                    AnyAtomicType::Double(d) => total += d.0,
                    other => {
                        return Err(ExpressionApplyError::new(format!(
                            "fn:avg: non-numeric value {:?}",
                            other
                        )));
                    }
                }
            }
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Double(ordered_float::OrderedFloat(
                    total / atoms.len() as f64
                ))
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-max
        "max" => func_min_max(args, context, false).map(Some),
        // https://www.w3.org/TR/xpath-functions-31/#func-min
        "min" => func_min_max(args, context, true).map(Some),
        _ => Ok(None),
    }
}

/// https://developer.mozilla.org/en-US/docs/Web/XPath/Functions/contains
fn func_contains<'tree>(
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    if args.len() != 2 {
        return Err(ExpressionApplyError {
            msg: format!(
                "contains: function expects 2 arguments, got {}",
                args.len()
            ),
        });
    }

    let arg1_set = &args[0];
    if arg1_set.len() > 1 {
        return Err(ExpressionApplyError {
            msg: format!(
                "contains: unexpected item set length {} for first argument",
                arg1_set.len()
            ),
        });
    }

    let haystack = if arg1_set.len() == 0 {
        String::from("")
    } else {
        func_string(&arg1_set[0], &context.item_tree)
    };

    let arg2_set = &args[1];
    if arg2_set.len() > 1 {
        return Err(ExpressionApplyError {
            msg: format!(
                "contains: unexpected item set length {} for second argument",
                arg2_set.len()
            ),
        });
    }

    let needle = if arg2_set.len() == 0 {
        String::from("")
    } else {
        func_string(&arg2_set[0], &context.item_tree)
    };

    Ok(xpath_item_set![XpathItem::AnyAtomicType(
        AnyAtomicType::Boolean(haystack.contains(&needle))
    )])
}

/// https://www.w3.org/TR/2017/REC-xpath-31-20170321/#dt-atomization
pub(crate) fn func_data<'tree>(
    set: &XpathItemSet<'tree>,
    item_tree: &'tree XpathItemTree,
) -> Vec<AnyAtomicType> {
    fn atomize<'tree>(item: &XpathItem, item_tree: &'tree XpathItemTree) -> AnyAtomicType {
        match item {
            XpathItem::Node(node) => match node {
                XpathItemTreeNode::DocumentNode(_) => {
                    AnyAtomicType::String(node.text_content(item_tree))
                }
                XpathItemTreeNode::ElementNode(_) => {
                    AnyAtomicType::String(node.text_content(item_tree))
                }
                XpathItemTreeNode::PINode(_) => AnyAtomicType::String(String::new()),
                XpathItemTreeNode::CommentNode(c) => {
                    AnyAtomicType::String(c.content.clone())
                }
                XpathItemTreeNode::TextNode(text) => AnyAtomicType::String(text.content.clone()),
                &XpathItemTreeNode::AttributeNode(attribute) => {
                    AnyAtomicType::String(attribute.value.clone())
                }
                XpathItemTreeNode::DoctypeNode(_) => AnyAtomicType::String(String::new()),
            },
            XpathItem::Function(_) => {
                // TODO: Per XPath 3.1, this should raise err:FOTY0013.
                // Returning a placeholder because func_data's signature doesn't support errors.
                AnyAtomicType::String(String::from("[function item]"))
            }
            XpathItem::AnyAtomicType(atomic) => atomic.clone(),
        }
    }

    set.iter().map(|item| atomize(item, item_tree)).collect()
}

/// https://www.w3.org/TR/xpath-functions-31/#func-string
pub(crate) fn func_string<'tree>(item: &XpathItem, item_tree: &'tree XpathItemTree) -> String {
    match item {
        XpathItem::Node(node) => match node {
            XpathItemTreeNode::DocumentNode(_) => node.text_content(item_tree),
            XpathItemTreeNode::ElementNode(_) => node.text_content(item_tree),
            XpathItemTreeNode::PINode(_) => String::new(),
            XpathItemTreeNode::CommentNode(c) => c.content.clone(),
            XpathItemTreeNode::TextNode(text) => text.content.clone(),
            XpathItemTreeNode::AttributeNode(attribute) => attribute.value.clone(),
            XpathItemTreeNode::DoctypeNode(_) => String::new(),
        },
        XpathItem::AnyAtomicType(atomic) => match atomic {
            AnyAtomicType::Boolean(b) => b.to_string(),
            AnyAtomicType::Integer(n) => n.to_string(),
            AnyAtomicType::Float(n) => n.to_string(),
            AnyAtomicType::Double(n) => n.to_string(),
            AnyAtomicType::String(s) => s.clone(),
        },
        XpathItem::Function(_) => {
            // TODO: Per XPath 3.1, fn:string is not defined for function items
            // and should raise an error. Returning placeholder because the
            // signature doesn't support errors.
            String::from("[function item]")
        }
    }
}

/// Validate that `args` has exactly `expected` elements.
fn check_arity(
    name: &str,
    args: &[XpathItemSet],
    expected: usize,
) -> Result<(), ExpressionApplyError> {
    if args.len() != expected {
        Err(ExpressionApplyError::new(format!(
            "{} expects {} argument(s), got {}",
            name,
            expected,
            args.len()
        )))
    } else {
        Ok(())
    }
}

/// Extract a single numeric value as f64 from an argument.
fn extract_double(arg: &XpathItemSet, item_tree: &XpathItemTree) -> Result<f64, ExpressionApplyError> {
    if arg.is_empty() {
        return Ok(f64::NAN);
    }
    match &arg[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(n)) => Ok(*n as f64),
        XpathItem::AnyAtomicType(AnyAtomicType::Float(f)) => Ok(f.0 as f64),
        XpathItem::AnyAtomicType(AnyAtomicType::Double(d)) => Ok(d.0),
        other => {
            let s = func_string(other, item_tree);
            s.trim().parse::<f64>().map_err(|_| {
                ExpressionApplyError::new(format!("Cannot convert '{}' to a number", s))
            })
        }
    }
}

/// Apply a unary numeric operation, preserving the source type.
fn func_numeric_unary<'tree>(
    arg: &XpathItemSet<'tree>,
    int_op: impl Fn(i64) -> i64,
    float_op: impl Fn(f32) -> f32,
    double_op: impl Fn(f64) -> f64,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    if arg.is_empty() {
        return Ok(XpathItemSet::new());
    }
    if arg.len() != 1 {
        return Err(ExpressionApplyError::new(format!(
            "numeric function expects a single item, got {}",
            arg.len()
        )));
    }
    match &arg[0] {
        XpathItem::AnyAtomicType(AnyAtomicType::Integer(n)) => Ok(xpath_item_set![
            XpathItem::AnyAtomicType(AnyAtomicType::Integer(int_op(*n)))
        ]),
        XpathItem::AnyAtomicType(AnyAtomicType::Float(f)) => Ok(xpath_item_set![
            XpathItem::AnyAtomicType(AnyAtomicType::Float(ordered_float::OrderedFloat(
                float_op(f.0)
            )))
        ]),
        XpathItem::AnyAtomicType(AnyAtomicType::Double(d)) => Ok(xpath_item_set![
            XpathItem::AnyAtomicType(AnyAtomicType::Double(ordered_float::OrderedFloat(
                double_op(d.0)
            )))
        ]),
        other => Err(ExpressionApplyError::new(format!(
            "numeric function requires a numeric argument, got {:?}",
            other
        ))),
    }
}

/// https://www.w3.org/TR/xpath-functions-31/#func-sum
fn func_sum<'tree>(
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    if args.is_empty() || args.len() > 2 {
        return Err(ExpressionApplyError::new(format!(
            "fn:sum expects 1 or 2 arguments, got {}",
            args.len()
        )));
    }
    if args[0].is_empty() {
        // Return $zero or 0 if the sequence is empty.
        if args.len() == 2 {
            return Ok(args[1].clone());
        }
        return Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Integer(0)
        )]);
    }
    let atoms = func_data(&args[0], context.item_tree);
    let mut total: f64 = 0.0;
    let mut all_integers = true;
    for atom in &atoms {
        match atom {
            AnyAtomicType::Integer(n) => total += *n as f64,
            AnyAtomicType::Float(f) => {
                total += f.0 as f64;
                all_integers = false;
            }
            AnyAtomicType::Double(d) => {
                total += d.0;
                all_integers = false;
            }
            other => {
                return Err(ExpressionApplyError::new(format!(
                    "fn:sum: non-numeric value {:?}",
                    other
                )));
            }
        }
    }
    if all_integers {
        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Integer(total as i64)
        )])
    } else {
        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Double(ordered_float::OrderedFloat(total))
        )])
    }
}

/// Build a `regex::Regex` from an XPath pattern string and flags.
///
/// Supported flags: `i` (case-insensitive), `s` (dot-all), `m` (multi-line), `x` (extended).
fn build_regex(pattern: &str, flags: &str) -> Result<regex::Regex, ExpressionApplyError> {
    let mut regex_pattern = String::new();
    if !flags.is_empty() {
        regex_pattern.push_str("(?");
        for ch in flags.chars() {
            match ch {
                'i' | 's' | 'm' | 'x' => regex_pattern.push(ch),
                _ => {
                    return Err(ExpressionApplyError::new(format!(
                        "unsupported regex flag '{}'",
                        ch
                    )));
                }
            }
        }
        regex_pattern.push(')');
    }
    regex_pattern.push_str(pattern);
    regex::Regex::new(&regex_pattern).map_err(|e| {
        ExpressionApplyError::new(format!("invalid regex pattern '{}': {}", pattern, e))
    })
}

/// Shared implementation for fn:min and fn:max.
fn func_min_max<'tree>(
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
    is_min: bool,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    check_arity(if is_min { "fn:min" } else { "fn:max" }, args, 1)?;
    if args[0].is_empty() {
        return Ok(XpathItemSet::new());
    }
    let atoms = func_data(&args[0], context.item_tree);
    let mut best: f64 = if is_min { f64::INFINITY } else { f64::NEG_INFINITY };
    let mut all_integers = true;
    for atom in &atoms {
        let val = match atom {
            AnyAtomicType::Integer(n) => *n as f64,
            AnyAtomicType::Float(f) => {
                all_integers = false;
                f.0 as f64
            }
            AnyAtomicType::Double(d) => {
                all_integers = false;
                d.0
            }
            AnyAtomicType::String(s) => {
                // String comparison: return the lexicographically min/max string.
                // Switch to string comparison mode.
                let mut best_str = s.as_str();
                for other in &atoms {
                    if let AnyAtomicType::String(os) = other {
                        if (is_min && os.as_str() < best_str)
                            || (!is_min && os.as_str() > best_str)
                        {
                            best_str = os.as_str();
                        }
                    }
                }
                return Ok(xpath_item_set![XpathItem::AnyAtomicType(
                    AnyAtomicType::String(best_str.to_string())
                )]);
            }
            other => {
                return Err(ExpressionApplyError::new(format!(
                    "fn:{}: non-comparable value {:?}",
                    if is_min { "min" } else { "max" },
                    other
                )));
            }
        };
        if is_min {
            if val < best {
                best = val;
            }
        } else if val > best {
            best = val;
        }
    }
    if all_integers {
        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Integer(best as i64)
        )])
    } else {
        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Double(ordered_float::OrderedFloat(best))
        )])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn function_call_should_parse() {
        // arrange
        let input = "my:three-argument-function(1,2,3)";

        // act
        let (next_input, res) = function_call(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "my:three-argument-function(1, 2, 3)");
    }

    #[test]
    fn function_call_should_parse_whitespace() {
        // arrange
        let input = "my:three-argument-function ( 1, 2, 3 )";

        // act
        let (next_input, res) = function_call(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "my:three-argument-function(1, 2, 3)");
    }
}
