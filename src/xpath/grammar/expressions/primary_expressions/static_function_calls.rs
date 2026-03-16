//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-context-item-expression>

use std::collections::HashSet;
use std::fmt::Display;

use nom::error::context;

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, Function, XpathItem},
            expressions::{
                common::{argument_list, Argument, ArgumentList},
                expr,
                postfix_expressions::invoke_function_item,
            },
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
            let body_parsed = expr(&body_source).ok().map(|(_, e)| Box::new(e));
            return Ok(xpath_item_set![XpathItem::Function(Function::Inline {
                params,
                body_source,
                body: body_parsed,
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
const XPATH_MAP_NS: &str = "http://www.w3.org/2005/xpath-functions/map";
const XPATH_ARRAY_NS: &str = "http://www.w3.org/2005/xpath-functions/array";
const XPATH_MATH_NS: &str = "http://www.w3.org/2005/xpath-functions/math";

pub(crate) fn dispatch_function<'tree>(
    name: &EQName,
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    // Determine namespace and local name for dispatch.
    enum FnNamespace<'a> {
        Default(&'a str), // fn: or unprefixed
        Map(&'a str),
        Array(&'a str),
        Math(&'a str),
    }

    let resolved = match name {
        EQName::QName(qname) => match qname {
            QName::PrefixedName(p) if p.prefix == "fn" => {
                Some(FnNamespace::Default(&p.local_part))
            }
            QName::PrefixedName(p) if p.prefix == "map" => {
                Some(FnNamespace::Map(&p.local_part))
            }
            QName::PrefixedName(p) if p.prefix == "array" => {
                Some(FnNamespace::Array(&p.local_part))
            }
            QName::PrefixedName(p) if p.prefix == "math" => {
                Some(FnNamespace::Math(&p.local_part))
            }
            QName::PrefixedName(_) => None,
            QName::UnprefixedName(n) => Some(FnNamespace::Default(n.as_str())),
        },
        EQName::UriQualifiedName(uqn) if uqn.uri == XPATH_FUNCTIONS_NS => {
            Some(FnNamespace::Default(&uqn.name))
        }
        EQName::UriQualifiedName(uqn) if uqn.uri == XPATH_MAP_NS => {
            Some(FnNamespace::Map(&uqn.name))
        }
        EQName::UriQualifiedName(uqn) if uqn.uri == XPATH_ARRAY_NS => {
            Some(FnNamespace::Array(&uqn.name))
        }
        EQName::UriQualifiedName(uqn) if uqn.uri == XPATH_MATH_NS => {
            Some(FnNamespace::Math(&uqn.name))
        }
        EQName::UriQualifiedName(_) => None,
    };

    match resolved {
        Some(FnNamespace::Default(local)) => {
            if let Some(result) = dispatch_by_local_name(local, args, context)? {
                return Ok(result);
            }
        }
        Some(FnNamespace::Map(local)) => {
            if let Some(result) = dispatch_map_function(local, args, context)? {
                return Ok(result);
            }
        }
        Some(FnNamespace::Array(local)) => {
            if let Some(result) = dispatch_array_function(local, args, context)? {
                return Ok(result);
            }
        }
        Some(FnNamespace::Math(local)) => {
            if let Some(result) = dispatch_math_function(local, args, context)? {
                return Ok(result);
            }
        }
        None => {}
    }

    Err(ExpressionApplyError::new(format!(
        "err:XPST0017: unknown function '{}'",
        name
    )))
}

/// Dispatch a function by its local name. Returns `Ok(Some(result))` if the
/// function is known, `Ok(None)` if unrecognized, or `Err` on evaluation error.
///
/// NOTE: When adding a new function here, also add it to [`is_known_fn_function`]
/// so that `fn:function-lookup` can find it.
fn dispatch_by_local_name<'tree>(
    local_name: &str,
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
) -> Result<Option<XpathItemSet<'tree>>, ExpressionApplyError> {
    match local_name {
        "root" => {
            check_arity_range("fn:root", args, 0, 1)?;
            if !args.is_empty() && !args[0].is_empty() {
                // Validate argument is a node.
                if !args[0].iter().all(|item| matches!(item, XpathItem::Node(_))) {
                    return Err(ExpressionApplyError::new(
                        "err:XPTY0004 fn:root: argument must be a node".to_string(),
                    ));
                }
            }
            Ok(Some(
                xpath_item_set![XpathItem::Node(context.item_tree.root())],
            ))
        }
        "contains" => func_contains(args, context).map(Some),
        "data" => {
            let target = if args.is_empty() {
                xpath_item_set![context.item.clone()]
            } else {
                args[0].clone()
            };
            let atoms = func_data(&target, context.item_tree)?;
            Ok(Some(
                atoms
                    .into_iter()
                    .map(|a| XpathItem::AnyAtomicType(a))
                    .collect(),
            ))
        }
        "string" => {
            if !args.is_empty() && args[0].is_empty() {
                // XPath 3.1 spec: if $arg is the empty sequence, return zero-length string
                return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                    AnyAtomicType::String(String::new())
                )]));
            }
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
            let s = func_string(target, context.item_tree)?;
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
            let ebv = args[0].boolean()?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(!ebv)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-boolean
        "boolean" => {
            check_arity("fn:boolean", args, 1)?;
            let ebv = args[0].boolean()?;
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
            // Handle boolean-to-number directly per XPath spec (true→1.0, false→0.0).
            let val = match target {
                XpathItem::AnyAtomicType(AnyAtomicType::Boolean(b)) => {
                    if *b { 1.0 } else { 0.0 }
                }
                _ => {
                    let s = func_string(target, context.item_tree)?;
                    s.trim().parse::<f64>().unwrap_or(f64::NAN)
                }
            };
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
            if args.len() == 2 {
                // fn:round($arg, $precision)
                let raw_precision = extract_double(&args[1], context.item_tree)?;
                let precision = (raw_precision.round() as i64).clamp(-20, 20) as i32;
                let factor = 10f64.powi(precision);
                func_numeric_unary(
                    &args[0],
                    |n| round_integer(n, precision),
                    |f| ((f as f64 * factor).round() / factor) as f32,
                    |d| (d * factor).round() / factor,
                )
                .map(Some)
            } else {
                check_arity("fn:round", args, 1)?;
                func_numeric_unary(&args[0], |i| i, |f| f.round(), |d| d.round()).map(Some)
            }
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
                    if arg.len() > 1 {
                        return Err(ExpressionApplyError::new(
                            "err:XPTY0004: fn:concat argument must be a single item or empty sequence".to_string(),
                        ));
                    }
                    result.push_str(&func_string(&arg[0], context.item_tree)?);
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
                extract_string_arg(&args[1], context.item_tree)?
            } else {
                String::new()
            };
            let parts: Vec<String> = args[0]
                .iter()
                .map(|item| func_string(item, context.item_tree))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(parts.join(&separator))
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-string-length
        "string-length" => {
            let s = if args.is_empty() {
                func_string(&context.item, context.item_tree)?
            } else {
                check_arity("fn:string-length", args, 1)?;
                extract_string_arg(&args[0], context.item_tree)?
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Integer(s.chars().count() as i64)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-normalize-space
        "normalize-space" => {
            let s = if args.is_empty() {
                func_string(&context.item, context.item_tree)?
            } else {
                check_arity("fn:normalize-space", args, 1)?;
                extract_string_arg(&args[0], context.item_tree)?
            };
            let normalized = s.split_whitespace().collect::<Vec<_>>().join(" ");
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(normalized)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-upper-case
        "upper-case" => {
            check_arity("fn:upper-case", args, 1)?;
            let s = extract_string_arg(&args[0], context.item_tree)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(s.to_uppercase())
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-lower-case
        "lower-case" => {
            check_arity("fn:lower-case", args, 1)?;
            let s = extract_string_arg(&args[0], context.item_tree)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(s.to_lowercase())
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-starts-with
        "starts-with" => {
            check_arity("fn:starts-with", args, 2)?;
            let haystack = extract_string_arg(&args[0], context.item_tree)?;
            let needle = extract_string_arg(&args[1], context.item_tree)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(haystack.starts_with(&needle))
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-ends-with
        "ends-with" => {
            check_arity("fn:ends-with", args, 2)?;
            let haystack = extract_string_arg(&args[0], context.item_tree)?;
            let needle = extract_string_arg(&args[1], context.item_tree)?;
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
            let s = extract_string_arg(&args[0], context.item_tree)?;
            let chars: Vec<char> = s.chars().collect();
            // XPath uses 1-based indexing with rounding.
            let start_double = extract_double(&args[1], context.item_tree)?;
            if start_double.is_nan() {
                return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                    AnyAtomicType::String(String::new())
                )]));
            }
            let result = if args.len() == 3 {
                let len_double = extract_double(&args[2], context.item_tree)?;
                if len_double.is_nan() {
                    return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::String(String::new())
                    )]));
                }
                if start_double.is_infinite() && start_double < 0.0 {
                    if len_double.is_infinite() && len_double > 0.0 {
                        // Per XPath spec: substring("...", -INF, INF) returns the full string.
                        return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                            AnyAtomicType::String(chars.iter().collect())
                        )]));
                    }
                    // negative infinity start with finite length: result is empty.
                    return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::String(String::new())
                    )]));
                }
                // Perform 1-based to 0-based conversion in f64 space to avoid
                // i64 overflow on extreme finite values (e.g. 1e19).
                let end_f64 = start_double.round() + len_double.round() - 1.0;
                let end = if end_f64 <= 0.0 {
                    0
                } else if end_f64 >= chars.len() as f64 {
                    chars.len()
                } else {
                    end_f64 as usize
                };
                let start_f64 = start_double.round() - 1.0;
                let start = if start_f64 <= 0.0 {
                    0
                } else if start_f64 >= chars.len() as f64 {
                    chars.len()
                } else {
                    start_f64 as usize
                };
                if start >= end {
                    String::new()
                } else {
                    chars[start..end].iter().collect()
                }
            } else {
                if start_double == f64::NEG_INFINITY {
                    // 2-arg form with -INF start: all chars are included.
                    return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::String(chars.iter().collect())
                    )]));
                }
                let start_f64 = start_double.round() - 1.0;
                let start = if start_f64 <= 0.0 {
                    0
                } else if start_f64 >= chars.len() as f64 {
                    chars.len()
                } else {
                    start_f64 as usize
                };
                chars[start..].iter().collect()
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-substring-before
        "substring-before" => {
            check_arity("fn:substring-before", args, 2)?;
            let s = extract_string_arg(&args[0], context.item_tree)?;
            let sub = extract_string_arg(&args[1], context.item_tree)?;
            let result = s.find(&sub).map(|i| &s[..i]).unwrap_or("").to_string();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-substring-after
        "substring-after" => {
            check_arity("fn:substring-after", args, 2)?;
            let s = extract_string_arg(&args[0], context.item_tree)?;
            let sub = extract_string_arg(&args[1], context.item_tree)?;
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
            let s = extract_string_arg(&args[0], context.item_tree)?;
            let map_from: Vec<char> = extract_string_arg(&args[1], context.item_tree)?.chars().collect();
            let map_to: Vec<char> = extract_string_arg(&args[2], context.item_tree)?.chars().collect();
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
            let atoms = func_data(&args[0], context.item_tree)?;
            let mut seen: Vec<AnyAtomicType> = Vec::new();
            let mut result = XpathItemSet::new();
            for a in atoms {
                if !seen.iter().any(|existing| atoms_equal(existing, &a)) {
                    seen.push(a.clone());
                    result.insert(XpathItem::AnyAtomicType(a));
                }
            }
            Ok(Some(result))
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
                    XpathItemTreeNode::PINode(pi) => pi.target.clone(),
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
                    XpathItemTreeNode::PINode(pi) => pi.target.clone(),
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
            let input = extract_string_arg(&args[0], context.item_tree)?;
            let pattern = extract_string_arg(&args[1], context.item_tree)?;
            let flags = if args.len() == 3 {
                extract_string_arg(&args[2], context.item_tree)?
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
            let input = extract_string_arg(&args[0], context.item_tree)?;
            let pattern = extract_string_arg(&args[1], context.item_tree)?;
            let replacement = extract_string_arg(&args[2], context.item_tree)?;
            let flags = if args.len() == 4 {
                extract_string_arg(&args[3], context.item_tree)?
            } else {
                String::new()
            };
            let re = build_regex(&pattern, &flags)?;
            // Convert XPath replacement syntax (\1, \2, ...) to regex crate syntax ($1, $2, ...)
            // and escape literal $ signs that have no special meaning in XPath.
            let converted_replacement = xpath_replacement_to_regex(&replacement);
            let result = re.replace_all(&input, converted_replacement.as_str()).to_string();
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
            let input = extract_string_arg(&args[0], context.item_tree)?;
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
            let pattern = extract_string_arg(&args[1], context.item_tree)?;
            let flags = if args.len() == 3 {
                extract_string_arg(&args[2], context.item_tree)?
            } else {
                String::new()
            };
            let re = build_regex(&pattern, &flags)?;
            let tokens: XpathItemSet = re
                .split(&input)
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
            // NaN start → empty sequence.
            if start_double.is_nan() {
                return Ok(Some(XpathItemSet::new()));
            }
            // +Infinity start → empty sequence (nothing starts at infinity).
            if start_double.is_infinite() && start_double > 0.0 {
                return Ok(Some(XpathItemSet::new()));
            }
            let seq_len = args[0].len();
            if args.len() == 3 {
                let len_double = extract_double(&args[2], context.item_tree)?;
                // NaN length → empty sequence.
                if len_double.is_nan() {
                    return Ok(Some(XpathItemSet::new()));
                }
                // Compute start_rounded and end_f as f64 to avoid overflow on cast.
                let start_rounded = start_double.round();
                let end_f = start_rounded + len_double.round();
                // NaN end (e.g. -INF + INF) → empty sequence.
                if end_f.is_nan() {
                    return Ok(Some(XpathItemSet::new()));
                }
                let start = if start_rounded.is_infinite() && start_rounded < 0.0 {
                    0
                } else {
                    (start_rounded as i64 - 1).max(0) as usize
                };
                let start = start.min(seq_len);
                let end = if end_f.is_infinite() && end_f > 0.0 {
                    seq_len
                } else if end_f < 1.0 {
                    0
                } else {
                    ((end_f as i64) - 1).max(0) as usize
                };
                let end = end.min(seq_len);
                if start >= end {
                    Ok(Some(XpathItemSet::new()))
                } else {
                    Ok(Some(args[0].iter().skip(start).take(end - start).cloned().collect()))
                }
            } else {
                // -Infinity start (2-arg) → return full sequence.
                if start_double.is_infinite() && start_double < 0.0 {
                    return Ok(Some(args[0].clone()));
                }
                let start = (start_double.round() as i64 - 1).max(0) as usize;
                let start = start.min(seq_len);
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
            // Atomize both the sequence and the search value before comparing.
            let seq_atoms = func_data(&args[0], context.item_tree)?;
            let search_atoms = func_data(&args[1], context.item_tree)?;
            let search_atom = &search_atoms[0];
            let positions: XpathItemSet = seq_atoms
                .iter()
                .enumerate()
                .filter(|(_, atom)| atoms_equal(atom, search_atom))
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
            let atoms = func_data(&args[0], context.item_tree)?;
            let mut total_f64: f64 = 0.0;
            let mut total_i64: i64 = 0;
            let mut all_integers = true;
            for atom in &atoms {
                match atom {
                    AnyAtomicType::Integer(n) => {
                        if all_integers {
                            match total_i64.checked_add(*n) {
                                Some(v) => total_i64 = v,
                                None => all_integers = false,
                            }
                        }
                        total_f64 += *n as f64;
                    }
                    AnyAtomicType::Float(f) => {
                        all_integers = false;
                        total_f64 += f.0 as f64;
                    }
                    AnyAtomicType::Double(d) => {
                        all_integers = false;
                        total_f64 += d.0;
                    }
                    AnyAtomicType::String(s) => {
                        all_integers = false;
                        // xs:untypedAtomic -> cast to xs:double per XPath 3.1 F&O 15.4.5
                        let d: f64 = s.trim().parse().map_err(|_| {
                            ExpressionApplyError::new(format!(
                                "fn:avg: cannot cast {:?} to xs:double",
                                s
                            ))
                        })?;
                        total_f64 += d;
                    }
                    other => {
                        return Err(ExpressionApplyError::new(format!(
                            "fn:avg: non-numeric value {:?}",
                            other
                        )));
                    }
                }
            }
            let count = atoms.len() as i64;
            if all_integers && total_i64 % count == 0 {
                Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                    AnyAtomicType::Integer(total_i64 / count)
                )]))
            } else {
                Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                    AnyAtomicType::Double(ordered_float::OrderedFloat(
                        total_f64 / atoms.len() as f64
                    ))
                )]))
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-max
        "max" => func_min_max(args, context, false).map(Some),
        // https://www.w3.org/TR/xpath-functions-31/#func-min
        "min" => func_min_max(args, context, true).map(Some),
        // Additional numeric functions
        // https://www.w3.org/TR/xpath-functions-31/#func-round-half-to-even
        "round-half-to-even" => {
            if args.is_empty() || args.len() > 2 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:round-half-to-even expects 1 or 2 arguments, got {}",
                    args.len()
                )));
            }
            let precision = if args.len() == 2 {
                let raw = extract_double(&args[1], context.item_tree)?;
                (raw.round() as i64).clamp(-20, 20) as i32
            } else {
                0
            };
            let factor = 10f64.powi(precision);
            func_numeric_unary(
                &args[0],
                |n| round_half_to_even_integer(n, precision),
                |f| {
                    let v = (f as f64 * factor as f64).round_ties_even() / factor as f64;
                    v as f32
                },
                |d| (d * factor as f64).round_ties_even() / factor as f64,
            )
            .map(Some)
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-format-integer
        "format-integer" => {
            check_arity("fn:format-integer", args, 2)?;
            let n_f64 = extract_double(&args[0], context.item_tree)?;
            if n_f64.is_nan() || n_f64.is_infinite() || n_f64 != n_f64.trunc() || n_f64 > i64::MAX as f64 || n_f64 < i64::MIN as f64 {
                return Err(ExpressionApplyError::new(
                    "err:XPTY0004 format-integer: argument is not a valid integer".to_string()
                ));
            }
            let n = n_f64 as i64;
            let picture = extract_string_arg(&args[1], context.item_tree)?;
            let result = match picture.as_str() {
                "1" => n.to_string(),
                "01" => format!("{:02}", n),
                "001" => format!("{:03}", n),
                "a" => {
                    if n >= 1 {
                        format_alpha(n, b'a')
                    } else {
                        n.to_string()
                    }
                }
                "A" => {
                    if n >= 1 {
                        format_alpha(n, b'A')
                    } else {
                        n.to_string()
                    }
                }
                "i" => func_to_roman(n).map(|r| r.to_lowercase()).unwrap_or_else(|| n.to_string()),
                "I" => func_to_roman(n).unwrap_or_else(|| n.to_string()),
                "w" => func_number_to_words(n).to_lowercase(),
                "W" => func_number_to_words(n).to_uppercase(),
                _ => n.to_string(),
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // Additional string functions
        // https://www.w3.org/TR/xpath-functions-31/#func-compare
        "compare" => {
            check_arity("fn:compare", args, 2)?;
            if args[0].is_empty() || args[1].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            let a = func_string(&args[0][0], context.item_tree)?;
            let b = func_string(&args[1][0], context.item_tree)?;
            let result = match a.cmp(&b) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Integer(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-codepoint-equal
        "codepoint-equal" => {
            check_arity("fn:codepoint-equal", args, 2)?;
            if args[0].is_empty() || args[1].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            let a = func_string(&args[0][0], context.item_tree)?;
            let b = func_string(&args[1][0], context.item_tree)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(a == b)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-codepoints-to-string
        "codepoints-to-string" => {
            check_arity("fn:codepoints-to-string", args, 1)?;
            let mut result = String::new();
            for item in args[0].iter() {
                let atoms = func_data(
                    &xpath_item_set![item.clone()],
                    context.item_tree,
                )?;
                for atom in atoms {
                    match atom {
                        AnyAtomicType::Integer(n) => {
                            if n < 0 {
                                return Err(ExpressionApplyError::new(format!(
                                    "fn:codepoints-to-string: invalid codepoint {}",
                                    n
                                )));
                            }
                            let ch = char::from_u32(n as u32).ok_or_else(|| {
                                ExpressionApplyError::new(format!(
                                    "fn:codepoints-to-string: invalid codepoint {}",
                                    n
                                ))
                            })?;
                            result.push(ch);
                        }
                        _ => {
                            return Err(ExpressionApplyError::new(
                                "fn:codepoints-to-string: expected integer codepoints"
                                    .to_string(),
                            ));
                        }
                    }
                }
            }
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-string-to-codepoints
        "string-to-codepoints" => {
            check_arity("fn:string-to-codepoints", args, 1)?;
            if args[0].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            let s = func_string(&args[0][0], context.item_tree)?;
            if s.is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            let codepoints: XpathItemSet = s
                .chars()
                .map(|c| XpathItem::AnyAtomicType(AnyAtomicType::Integer(c as i64)))
                .collect();
            Ok(Some(codepoints))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-encode-for-uri
        "encode-for-uri" => {
            check_arity("fn:encode-for-uri", args, 1)?;
            let s = extract_string_arg(&args[0], context.item_tree)?;
            let encoded: String = s
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || "-._~".contains(c) {
                        c.to_string()
                    } else {
                        let mut buf = [0u8; 4];
                        c.encode_utf8(&mut buf);
                        buf[..c.len_utf8()]
                            .iter()
                            .map(|b| format!("%{:02X}", b))
                            .collect()
                    }
                })
                .collect();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(encoded)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-iri-to-uri
        "iri-to-uri" => {
            check_arity("fn:iri-to-uri", args, 1)?;
            let s = extract_string_arg(&args[0], context.item_tree)?;
            // Encode only non-ASCII and disallowed URI characters; preserve
            // characters that are valid in a URI (including %, /, ?, #, etc.).
            let encoded: String = s
                .chars()
                .map(|c| {
                    if c.is_ascii() && !c.is_ascii_control() && c != ' ' {
                        c.to_string()
                    } else {
                        let mut buf = [0u8; 4];
                        c.encode_utf8(&mut buf);
                        buf[..c.len_utf8()]
                            .iter()
                            .map(|b| format!("%{:02X}", b))
                            .collect()
                    }
                })
                .collect();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(encoded)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-escape-html-uri
        "escape-html-uri" => {
            check_arity("fn:escape-html-uri", args, 1)?;
            let s = extract_string_arg(&args[0], context.item_tree)?;
            // Escape characters outside the printable ASCII range (0x20-0x7E).
            let encoded: String = s
                .chars()
                .map(|c| {
                    if c as u32 >= 0x20 && c as u32 <= 0x7E {
                        c.to_string()
                    } else {
                        let mut buf = [0u8; 4];
                        c.encode_utf8(&mut buf);
                        buf[..c.len_utf8()]
                            .iter()
                            .map(|b| format!("%{:02X}", b))
                            .collect()
                    }
                })
                .collect();
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(encoded)
            )]))
        }
        // Additional sequence functions
        // https://www.w3.org/TR/xpath-functions-31/#func-deep-equal
        "deep-equal" => {
            check_arity("fn:deep-equal", args, 2)?;
            let mut equal = args[0].len() == args[1].len();
            if equal {
                for (a, b) in args[0].iter().zip(args[1].iter()) {
                    equal = deep_equal_items(a, b, context.item_tree)?;
                    if !equal {
                        break;
                    }
                }
            }
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(equal)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-unordered
        "unordered" => {
            check_arity("fn:unordered", args, 1)?;
            Ok(Some(args[0].clone()))
        }
        // Node functions
        // https://www.w3.org/TR/xpath-functions-31/#func-has-children
        "has-children" => {
            let target = if args.is_empty() {
                &context.item
            } else {
                check_arity("fn:has-children", args, 1)?;
                if args[0].is_empty() {
                    return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::Boolean(false)
                    )]));
                }
                &args[0][0]
            };
            let has = match target {
                XpathItem::Node(node) => !node.children(context.item_tree).is_empty(),
                _ => false,
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(has)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-path
        "path" => {
            let target = if args.is_empty() {
                &context.item
            } else {
                check_arity("fn:path", args, 1)?;
                if args[0].is_empty() {
                    return Ok(Some(XpathItemSet::new()));
                }
                &args[0][0]
            };
            match target {
                XpathItem::Node(node) => {
                    let path = func_node_path(node, context.item_tree);
                    Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::String(path)
                    )]))
                }
                _ => Err(ExpressionApplyError::new(
                    "fn:path: argument is not a node".to_string(),
                )),
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-namespace-uri
        "namespace-uri" => {
            // HTML-only processor: namespace URI is always empty.
            if !args.is_empty() {
                check_arity("fn:namespace-uri", args, 1)?;
            }
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(String::new())
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-lang
        "lang" => {
            check_arity("fn:lang", args, 1)?;
            let test_lang = extract_string_arg(&args[0], context.item_tree)?;
            let test_lang_lower = test_lang.to_lowercase();
            // Walk up from context node looking for xml:lang or lang attribute.
            let mut result = false;
            if let XpathItem::Node(node) = &context.item {
                let mut current = Some(*node);
                while let Some(cur) = current {
                    if let XpathItemTreeNode::ElementNode(e) = cur {
                        let lang_attr = e.attributes(context.item_tree)
                            .into_iter()
                            .find(|a| a.name == "lang" || a.name == "xml:lang");
                        if let Some(attr) = lang_attr {
                            let lang_lower = attr.value.to_lowercase();
                            result = lang_lower == test_lang_lower
                                || lang_lower.starts_with(&format!("{}-", test_lang_lower));
                            break;
                        }
                    }
                    current = cur.parent(context.item_tree);
                }
            }
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(result)
            )]))
        }
        // Accessor functions
        // https://www.w3.org/TR/xpath-functions-31/#func-node-name
        "node-name" => {
            let target = if args.is_empty() {
                &context.item
            } else {
                check_arity("fn:node-name", args, 1)?;
                if args[0].is_empty() {
                    return Ok(Some(XpathItemSet::new()));
                }
                &args[0][0]
            };
            match target {
                XpathItem::Node(node) => {
                    let qname = match node {
                        XpathItemTreeNode::ElementNode(e) => Some(AnyAtomicType::QName {
                            namespace_uri: e.namespace.clone().unwrap_or_default(),
                            local_name: e.name.clone(),
                            prefix: None,
                        }),
                        XpathItemTreeNode::AttributeNode(a) => Some(AnyAtomicType::QName {
                            namespace_uri: String::new(),
                            local_name: a.name.clone(),
                            prefix: None,
                        }),
                        XpathItemTreeNode::PINode(pi) => Some(AnyAtomicType::QName {
                            namespace_uri: String::new(),
                            local_name: pi.target.clone(),
                            prefix: None,
                        }),
                        _ => None,
                    };
                    match qname {
                        Some(q) => Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(q)])),
                        None => Ok(Some(XpathItemSet::new())),
                    }
                }
                _ => Ok(Some(XpathItemSet::new())),
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-nilled
        "nilled" => {
            // HTML-only processor: elements are never nilled.
            if !args.is_empty() {
                check_arity("fn:nilled", args, 1)?;
            }
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(false)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-generate-id
        "generate-id" => {
            let target = if args.is_empty() {
                &context.item
            } else {
                check_arity("fn:generate-id", args, 1)?;
                if args[0].is_empty() {
                    return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::String(String::new())
                    )]));
                }
                &args[0][0]
            };
            let id = match target {
                XpathItem::Node(node) => {
                    if let Some(node_id) = node.node_id() {
                        format!("N{}", usize::from(node_id))
                    } else {
                        String::from("N0")
                    }
                }
                _ => String::new(),
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(id)
            )]))
        }
        // Higher-order functions
        // https://www.w3.org/TR/xpath-functions-31/#func-for-each
        "for-each" => {
            check_arity("fn:for-each", args, 2)?;
            let func = extract_function_item(&args[1], "fn:for-each")?;
            let mut result = XpathItemSet::new();
            for item in args[0].iter() {
                let call_result = invoke_function_item(
                    func,
                    vec![xpath_item_set![item.clone()]],
                    context,
                )?;
                for r in call_result.into_iter() {
                    result.insert(r);
                }
            }
            Ok(Some(result))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-filter
        "filter" => {
            check_arity("fn:filter", args, 2)?;
            let func = extract_function_item(&args[1], "fn:filter")?;
            let mut result = XpathItemSet::new();
            for item in args[0].iter() {
                let call_result = invoke_function_item(
                    func,
                    vec![xpath_item_set![item.clone()]],
                    context,
                )?;
                if call_result.boolean()? {
                    result.insert(item.clone());
                }
            }
            Ok(Some(result))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-fold-left
        "fold-left" => {
            check_arity("fn:fold-left", args, 3)?;
            let func = extract_function_item(&args[2], "fn:fold-left")?;
            let mut accumulator = args[1].clone();
            for item in args[0].iter() {
                accumulator = invoke_function_item(
                    func,
                    vec![accumulator, xpath_item_set![item.clone()]],
                    context,
                )?;
            }
            Ok(Some(accumulator))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-fold-right
        "fold-right" => {
            check_arity("fn:fold-right", args, 3)?;
            let func = extract_function_item(&args[2], "fn:fold-right")?;
            let items: Vec<_> = args[0].iter().cloned().collect();
            let mut accumulator = args[1].clone();
            for item in items.into_iter().rev() {
                accumulator = invoke_function_item(
                    func,
                    vec![xpath_item_set![item], accumulator],
                    context,
                )?;
            }
            Ok(Some(accumulator))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-for-each-pair
        "for-each-pair" => {
            check_arity("fn:for-each-pair", args, 3)?;
            let func = extract_function_item(&args[2], "fn:for-each-pair")?;
            let mut result = XpathItemSet::new();
            for (a, b) in args[0].iter().zip(args[1].iter()) {
                let call_result = invoke_function_item(
                    func,
                    vec![xpath_item_set![a.clone()], xpath_item_set![b.clone()]],
                    context,
                )?;
                for r in call_result.into_iter() {
                    result.insert(r);
                }
            }
            Ok(Some(result))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-sort
        "sort" => {
            if args.is_empty() || args.len() > 3 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:sort expects 1-3 arguments, got {}",
                    args.len()
                )));
            }
            let items: Vec<XpathItem> = args[0].iter().cloned().collect();
            // Spec: fn:sort($input, $collation?, $key?). Arg 2 is collation (ignored),
            // arg 3 is the key function.
            let key_func_arg = if args.len() == 3 {
                Some(&args[2])
            } else {
                None
            };
            if let Some(key_arg) = key_func_arg {
                let func = extract_function_item(key_arg, "fn:sort")?;
                let mut keyed: Vec<(XpathItem, Vec<AnyAtomicType>)> = Vec::new();
                for item in &items {
                    let key = invoke_function_item(
                        func,
                        vec![xpath_item_set![item.clone()]],
                        context,
                    )?;
                    let atoms = func_data(&key, context.item_tree)?;
                    keyed.push((item.clone(), atoms));
                }
                keyed.sort_by(|(_, a), (_, b)| sort_cmp_atomized(a, b));
                let result: XpathItemSet =
                    keyed.into_iter().map(|(item, _)| item).collect();
                Ok(Some(result))
            } else {
                // Sort by atomized value.
                let mut items_with_keys: Vec<(XpathItem, Vec<AnyAtomicType>)> = Vec::new();
                for item in items {
                    let atoms = func_data(
                        &xpath_item_set![item.clone()],
                        context.item_tree,
                    )?;
                    items_with_keys.push((item, atoms));
                }
                items_with_keys.sort_by(|(_, a), (_, b)| sort_cmp_atomized(a, b));
                Ok(Some(items_with_keys.into_iter().map(|(item, _)| item).collect()))
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-apply
        "apply" => {
            check_arity("fn:apply", args, 2)?;
            let func = extract_function_item(&args[0], "fn:apply")?;
            if args[1].is_empty() {
                return Err(ExpressionApplyError::new(
                    "fn:apply: second argument must be an array, got empty sequence".to_string(),
                ));
            }
            // Second argument must be an array — extract its members as individual arguments.
            let call_args: Vec<XpathItemSet> = match &args[1][0] {
                XpathItem::Function(Function::Array { members }) => {
                    members
                        .iter()
                        .map(|member| {
                            member
                                .iter()
                                .map(|v| XpathItem::AnyAtomicType(v.clone()))
                                .collect()
                        })
                        .collect()
                }
                _ => {
                    return Err(ExpressionApplyError::new(
                        "fn:apply: second argument must be an array".to_string(),
                    ));
                }
            };
            invoke_function_item(func, call_args, context).map(Some)
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-function-name
        "function-name" => {
            check_arity("fn:function-name", args, 1)?;
            if args[0].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            match &args[0][0] {
                XpathItem::Function(Function::Named { name, .. }) => {
                    // Parse "prefix:local" to extract prefix and local name.
                    let (prefix, local_name) = if let Some(colon_pos) = name.find(':') {
                        (
                            Some(name[..colon_pos].to_string()),
                            name[colon_pos + 1..].to_string(),
                        )
                    } else {
                        (Some("fn".to_string()), name.clone())
                    };
                    // Map known prefixes to namespace URIs.
                    let namespace_uri = match prefix.as_deref() {
                        Some("fn") => "http://www.w3.org/2005/xpath-functions".to_string(),
                        Some("math") => "http://www.w3.org/2005/xpath-functions/math".to_string(),
                        Some("map") => "http://www.w3.org/2005/xpath-functions/map".to_string(),
                        Some("array") => "http://www.w3.org/2005/xpath-functions/array".to_string(),
                        _ => String::new(),
                    };
                    Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::QName {
                            namespace_uri,
                            local_name,
                            prefix,
                        }
                    )]))
                }
                XpathItem::Function(_) => Ok(Some(XpathItemSet::new())),
                _ => Err(ExpressionApplyError::new(
                    "fn:function-name: argument is not a function".to_string(),
                )),
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-function-arity
        "function-arity" => {
            check_arity("fn:function-arity", args, 1)?;
            if args[0].is_empty() {
                return Err(ExpressionApplyError::new(
                    "fn:function-arity: argument is empty".to_string(),
                ));
            }
            match &args[0][0] {
                XpathItem::Function(Function::Named { arity, .. }) => {
                    Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::Integer(*arity as i64)
                    )]))
                }
                XpathItem::Function(Function::Inline { params, .. }) => {
                    Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::Integer(params.len() as i64)
                    )]))
                }
                XpathItem::Function(Function::Map { .. }) | XpathItem::Function(Function::Array { .. }) => {
                    Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::Integer(1)
                    )]))
                }
                _ => Err(ExpressionApplyError::new(
                    "fn:function-arity: argument is not a function".to_string(),
                )),
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-format-number
        "format-number" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:format-number expects 2-3 arguments, got {}",
                    args.len()
                )));
            }
            // Arg 3 (decimal-format-name) is ignored; only default format supported.
            let value = extract_double(&args[0], context.item_tree)?;
            if args[1].is_empty() {
                return Err(ExpressionApplyError::new(
                    "fn:format-number: picture argument must not be an empty sequence".to_string(),
                ));
            }
            let picture = func_string(&args[1][0], context.item_tree)?;
            let result = func_format_number(value, &picture)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-normalize-unicode
        "normalize-unicode" => {
            if args.is_empty() || args.len() > 2 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:normalize-unicode expects 1-2 arguments, got {}",
                    args.len()
                )));
            }
            if args[0].is_empty() {
                return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                    AnyAtomicType::String(String::new())
                )]));
            }
            let input = func_string(&args[0][0], context.item_tree)?;
            let form = if args.len() == 2 {
                let raw = extract_string_arg(&args[1], context.item_tree)?;
                if raw.is_empty() { "NFC".to_string() } else { raw }
                    .trim()
                    .to_uppercase()
            } else {
                "NFC".to_string()
            };
            let result = func_normalize_unicode(&input, &form)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(result)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-innermost
        "innermost" => {
            check_arity("fn:innermost", args, 1)?;
            let nodes: Vec<&XpathItemTreeNode> = args[0]
                .iter()
                .filter_map(|item| match item {
                    XpathItem::Node(n) => Some(*n),
                    _ => None,
                })
                .collect();
            let node_ids: Vec<Option<indextree::NodeId>> =
                nodes.iter().map(|n| n.node_id()).collect();
            let node_id_set: HashSet<indextree::NodeId> =
                node_ids.iter().filter_map(|id| *id).collect();
            let mut ancestor_ids: HashSet<indextree::NodeId> = HashSet::new();
            for id in node_id_set.iter() {
                for anc in id.ancestors(&context.item_tree.arena).skip(1) {
                    if node_id_set.contains(&anc) {
                        ancestor_ids.insert(anc);
                    }
                }
            }
            let mut result = XpathItemSet::new();
            for (i, node) in nodes.iter().enumerate() {
                let dominated = match node_ids[i] {
                    Some(id) => ancestor_ids.contains(&id),
                    None => false,
                };
                if !dominated {
                    result.insert(XpathItem::Node(node));
                }
            }
            Ok(Some(result))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-outermost
        "outermost" => {
            check_arity("fn:outermost", args, 1)?;
            let nodes: Vec<&XpathItemTreeNode> = args[0]
                .iter()
                .filter_map(|item| match item {
                    XpathItem::Node(n) => Some(*n),
                    _ => None,
                })
                .collect();
            let node_ids: Vec<Option<indextree::NodeId>> =
                nodes.iter().map(|n| n.node_id()).collect();
            let node_id_set: HashSet<indextree::NodeId> =
                node_ids.iter().filter_map(|id| *id).collect();
            let mut result = XpathItemSet::new();
            for (i, node) in nodes.iter().enumerate() {
                let has_ancestor_in_set = if let Some(my_id) = node_ids[i] {
                    my_id
                        .ancestors(&context.item_tree.arena)
                        .skip(1)
                        .any(|anc| node_id_set.contains(&anc))
                } else {
                    false
                };
                if !has_ancestor_in_set {
                    result.insert(XpathItem::Node(node));
                }
            }
            Ok(Some(result))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-base-uri
        "base-uri" => {
            if args.len() > 1 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:base-uri expects 0-1 arguments, got {}",
                    args.len()
                )));
            }
            // HTML-only processor: no base URI tracking, return empty string.
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::String(String::new())
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-document-uri
        "document-uri" => {
            if args.len() > 1 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:document-uri expects 0-1 arguments, got {}",
                    args.len()
                )));
            }
            // HTML-only processor: no document URI tracking, return empty sequence.
            Ok(Some(XpathItemSet::new()))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-error
        "error" => {
            if args.len() > 3 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:error expects 0-3 arguments, got {}",
                    args.len()
                )));
            }
            // Extract error description from arg 2 if present, otherwise use arg 1 as code.
            let msg = if args.len() >= 2 && !args[1].is_empty() {
                func_string(&args[1][0], context.item_tree)?
            } else if !args.is_empty() && !args[0].is_empty() {
                func_string(&args[0][0], context.item_tree)?
            } else {
                "err:FOER0000".to_string()
            };
            Err(ExpressionApplyError::new(msg))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-trace
        "trace" => {
            if args.is_empty() || args.len() > 2 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:trace expects 1-2 arguments, got {}",
                    args.len()
                )));
            }
            let label = if args.len() == 2 {
                extract_string_arg(&args[1], context.item_tree)?
            } else {
                String::new()
            };
            // Log the trace to stderr, then return the input unchanged.
            for item in args[0].iter() {
                let s = func_string(item, context.item_tree)?;
                if label.is_empty() {
                    log::trace!("[fn:trace] {}", s);
                } else {
                    log::trace!("[fn:trace] {}: {}", label, s);
                }
            }
            Ok(Some(args[0].clone()))
        }
        // ID functions
        // https://www.w3.org/TR/xpath-functions-31/#func-id
        // https://www.w3.org/TR/xpath-functions-31/#func-element-with-id
        "id" | "element-with-id" => {
            if args.is_empty() || args.len() > 2 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:{} expects 1-2 arguments, got {}",
                    local_name,
                    args.len()
                )));
            }
            // Collect target IDs by tokenizing each string arg on whitespace.
            let mut target_ids = std::collections::HashSet::new();
            for item in args[0].iter() {
                let s = func_string(item, context.item_tree)?;
                for token in s.split_whitespace() {
                    target_ids.insert(token.to_string());
                }
            }
            // Find elements with matching id attributes.
            let mut result = XpathItemSet::new();
            for node in context.item_tree.iter() {
                if let XpathItemTreeNode::ElementNode(e) = node {
                    if let Some(id_val) = e.get_attribute(context.item_tree, "id") {
                        if target_ids.contains(id_val) {
                            result.insert(XpathItem::Node(node));
                        }
                    }
                }
            }
            result.sort_by_document_order();
            Ok(Some(result))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-idref
        "idref" => {
            // For non-schema-aware processors, no nodes have the is-idrefs
            // property, so this always returns an empty sequence.
            if args.is_empty() || args.len() > 2 {
                return Err(ExpressionApplyError::new(format!(
                    "fn:idref expects 1-2 arguments, got {}",
                    args.len()
                )));
            }
            Ok(Some(XpathItemSet::new()))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-QName
        "QName" => {
            check_arity("fn:QName", args, 2)?;
            // $paramURI as xs:string? — namespace URI (empty string or empty seq = no namespace)
            let namespace_uri = extract_string_arg(&args[0], context.item_tree)?;
            // $paramQName as xs:string — lexical QName ("prefix:local" or "local")
            if args[1].is_empty() {
                return Err(ExpressionApplyError::new(
                    "fn:QName: second argument (lexical QName) must not be empty".to_string(),
                ));
            }
            let lexical = func_string(&args[1][0], context.item_tree)?;
            let (prefix, local_name) = if let Some(colon_pos) = lexical.find(':') {
                (
                    Some(lexical[..colon_pos].to_string()),
                    lexical[colon_pos + 1..].to_string(),
                )
            } else {
                (None, lexical)
            };
            // TODO: validate that prefix and local_name are valid NCNames per spec
            // (could reuse xml_names::nc_name parser).

            // Per spec: if prefix is present, namespace URI must not be empty.
            if prefix.is_some() && namespace_uri.is_empty() {
                return Err(ExpressionApplyError::new(
                    "err:FOCA0002 fn:QName: prefix provided but namespace URI is empty"
                        .to_string(),
                ));
            }
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::QName {
                    namespace_uri,
                    local_name,
                    prefix,
                }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-local-name-from-QName
        "local-name-from-QName" => {
            check_arity("fn:local-name-from-QName", args, 1)?;
            if args[0].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            match &args[0][0] {
                XpathItem::AnyAtomicType(AnyAtomicType::QName { local_name, .. }) => {
                    Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                        AnyAtomicType::String(local_name.clone())
                    )]))
                }
                _ => Err(ExpressionApplyError::new(
                    "err:XPTY0004 fn:local-name-from-QName: argument is not a QName".to_string(),
                )),
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-namespace-uri-from-QName
        "namespace-uri-from-QName" => {
            check_arity("fn:namespace-uri-from-QName", args, 1)?;
            if args[0].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            match &args[0][0] {
                XpathItem::AnyAtomicType(AnyAtomicType::QName {
                    namespace_uri, ..
                }) => Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                    AnyAtomicType::String(namespace_uri.clone())
                )])),
                _ => Err(ExpressionApplyError::new(
                    "err:XPTY0004 fn:namespace-uri-from-QName: argument is not a QName"
                        .to_string(),
                )),
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-prefix-from-QName
        "prefix-from-QName" => {
            check_arity("fn:prefix-from-QName", args, 1)?;
            if args[0].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            match &args[0][0] {
                XpathItem::AnyAtomicType(AnyAtomicType::QName { prefix, .. }) => {
                    match prefix {
                        Some(p) => Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                            AnyAtomicType::String(p.clone())
                        )])),
                        None => Ok(Some(XpathItemSet::new())),
                    }
                }
                _ => Err(ExpressionApplyError::new(
                    "err:XPTY0004 fn:prefix-from-QName: argument is not a QName".to_string(),
                )),
            }
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-function-lookup
        "function-lookup" => {
            check_arity("fn:function-lookup", args, 2)?;
            if args[0].is_empty() || args[1].is_empty() {
                return Err(ExpressionApplyError::new(
                    "err:XPTY0004 fn:function-lookup: arguments must not be empty sequences"
                        .to_string(),
                ));
            }
            // $name as xs:QName
            let (namespace_uri, local_name) = match &args[0][0] {
                XpathItem::AnyAtomicType(AnyAtomicType::QName {
                    namespace_uri,
                    local_name,
                    ..
                }) => (namespace_uri.as_str(), local_name.as_str()),
                _ => {
                    return Err(ExpressionApplyError::new(
                        "err:XPTY0004 fn:function-lookup: first argument must be xs:QName"
                            .to_string(),
                    ))
                }
            };
            // $arity as xs:integer
            let arity = match &args[1][0] {
                XpathItem::AnyAtomicType(AnyAtomicType::Integer(n)) => {
                    if *n < 0 {
                        return Err(ExpressionApplyError::new(
                            "err:XPTY0004 fn:function-lookup: arity must be non-negative"
                                .to_string(),
                        ));
                    }
                    *n as u32
                }
                other => {
                    let s = func_string(other, context.item_tree)?;
                    s.parse::<u32>().map_err(|_| {
                        ExpressionApplyError::new(format!(
                            "fn:function-lookup: cannot convert '{}' to integer",
                            s
                        ))
                    })?
                }
            };
            // Resolve namespace URI to prefix and check if function exists.
            let (prefix, is_known) = match namespace_uri {
                XPATH_FUNCTIONS_NS => ("fn", is_known_fn_function(local_name, arity)),
                XPATH_MAP_NS => ("map", is_known_map_function(local_name, arity)),
                XPATH_ARRAY_NS => ("array", is_known_array_function(local_name, arity)),
                XPATH_MATH_NS => ("math", is_known_math_function(local_name, arity)),
                _ => return Ok(Some(XpathItemSet::new())),
            };
            if is_known {
                let name = format!("{}:{}", prefix, local_name);
                Ok(Some(xpath_item_set![XpathItem::Function(
                    Function::Named { name, arity }
                )]))
            } else {
                Ok(Some(XpathItemSet::new()))
            }
        }
        _ => Ok(None),
    }
}

/// Check if a function with the given local name and arity is known in the `fn:` namespace.
fn is_known_fn_function(name: &str, arity: u32) -> bool {
    matches!(
        (name, arity),
        ("root", 0)
            | ("root", 1)
            | ("contains", 2)
            | ("data", 0)
            | ("data", 1)
            | ("string", 0)
            | ("string", 1)
            | ("true", 0)
            | ("false", 0)
            | ("not", 1)
            | ("boolean", 1)
            | ("number", 0)
            | ("number", 1)
            | ("abs", 1)
            | ("ceiling", 1)
            | ("floor", 1)
            | ("round", 1)
            | ("round", 2)
            | ("concat", 2..)
            | ("string-join", 1)
            | ("string-join", 2)
            | ("string-length", 0)
            | ("string-length", 1)
            | ("normalize-space", 0)
            | ("normalize-space", 1)
            | ("upper-case", 1)
            | ("lower-case", 1)
            | ("starts-with", 2)
            | ("ends-with", 2)
            | ("substring", 2)
            | ("substring", 3)
            | ("substring-before", 2)
            | ("substring-after", 2)
            | ("translate", 3)
            | ("empty", 1)
            | ("exists", 1)
            | ("count", 1)
            | ("head", 1)
            | ("tail", 1)
            | ("reverse", 1)
            | ("distinct-values", 1)
            | ("sum", 1)
            | ("sum", 2)
            | ("name", 0)
            | ("name", 1)
            | ("local-name", 0)
            | ("local-name", 1)
            | ("position", 0)
            | ("last", 0)
            | ("matches", 2)
            | ("matches", 3)
            | ("replace", 3)
            | ("replace", 4)
            | ("tokenize", 1)
            | ("tokenize", 2)
            | ("tokenize", 3)
            | ("subsequence", 2)
            | ("subsequence", 3)
            | ("insert-before", 3)
            | ("remove", 2)
            | ("index-of", 2)
            | ("zero-or-one", 1)
            | ("one-or-more", 1)
            | ("exactly-one", 1)
            | ("avg", 1)
            | ("max", 1)
            | ("min", 1)
            | ("round-half-to-even", 1)
            | ("round-half-to-even", 2)
            | ("format-integer", 2)
            | ("compare", 2)
            | ("codepoint-equal", 2)
            | ("codepoints-to-string", 1)
            | ("string-to-codepoints", 1)
            | ("encode-for-uri", 1)
            | ("iri-to-uri", 1)
            | ("escape-html-uri", 1)
            | ("deep-equal", 2)
            | ("unordered", 1)
            | ("has-children", 0)
            | ("has-children", 1)
            | ("path", 0)
            | ("path", 1)
            | ("namespace-uri", 0)
            | ("namespace-uri", 1)
            | ("lang", 1)
            | ("node-name", 0)
            | ("node-name", 1)
            | ("nilled", 0)
            | ("nilled", 1)
            | ("generate-id", 0)
            | ("generate-id", 1)
            | ("for-each", 2)
            | ("filter", 2)
            | ("fold-left", 3)
            | ("fold-right", 3)
            | ("for-each-pair", 3)
            | ("sort", 1)
            | ("sort", 2)
            | ("sort", 3)
            | ("apply", 2)
            | ("function-name", 1)
            | ("function-arity", 1)
            | ("format-number", 2)
            | ("format-number", 3)
            | ("normalize-unicode", 1)
            | ("normalize-unicode", 2)
            | ("innermost", 1)
            | ("outermost", 1)
            | ("base-uri", 0)
            | ("base-uri", 1)
            | ("document-uri", 0)
            | ("document-uri", 1)
            | ("error", 0)
            | ("error", 1)
            | ("error", 2)
            | ("error", 3)
            | ("trace", 1)
            | ("trace", 2)
            | ("id", 1)
            | ("id", 2)
            | ("element-with-id", 1)
            | ("element-with-id", 2)
            | ("idref", 1)
            | ("idref", 2)
            | ("QName", 2)
            | ("local-name-from-QName", 1)
            | ("namespace-uri-from-QName", 1)
            | ("prefix-from-QName", 1)
            | ("function-lookup", 2)
    )
}

/// Check if a function with the given local name and arity is known in the `map:` namespace.
fn is_known_map_function(name: &str, arity: u32) -> bool {
    matches!(
        (name, arity),
        ("size", 1)
            | ("keys", 1)
            | ("contains", 2)
            | ("get", 2)
            | ("put", 3)
            | ("entry", 2)
            | ("remove", 2)
            | ("merge", 1)
            | ("merge", 2)
            | ("for-each", 2)
            | ("find", 2)
    )
}

/// Check if a function with the given local name and arity is known in the `array:` namespace.
fn is_known_array_function(name: &str, arity: u32) -> bool {
    matches!(
        (name, arity),
        ("size", 1)
            | ("get", 2)
            | ("put", 3)
            | ("append", 2)
            | ("subarray", 2)
            | ("subarray", 3)
            | ("remove", 2)
            | ("insert-before", 3)
            | ("head", 1)
            | ("tail", 1)
            | ("reverse", 1)
            | ("join", 1)
            | ("flatten", 1)
            | ("for-each", 2)
            | ("filter", 2)
            | ("fold-left", 3)
            | ("fold-right", 3)
            | ("for-each-pair", 3)
            | ("sort", 1)
            | ("sort", 2)
            | ("sort", 3)
    )
}

/// Check if a function with the given local name and arity is known in the `math:` namespace.
fn is_known_math_function(name: &str, arity: u32) -> bool {
    matches!(
        (name, arity),
        ("pi", 0)
            | ("exp", 1)
            | ("exp10", 1)
            | ("log", 1)
            | ("log10", 1)
            | ("sqrt", 1)
            | ("sin", 1)
            | ("cos", 1)
            | ("tan", 1)
            | ("asin", 1)
            | ("acos", 1)
            | ("atan", 1)
            | ("pow", 2)
            | ("atan2", 2)
    )
}

/// Dispatch `map:*` functions.
///
/// NOTE: When adding a new function here, also add it to [`is_known_map_function`]
/// so that `fn:function-lookup` can find it.
fn dispatch_map_function<'tree>(
    local_name: &str,
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
) -> Result<Option<XpathItemSet<'tree>>, ExpressionApplyError> {
    match local_name {
        // https://www.w3.org/TR/xpath-functions-31/#func-map-size
        "size" => {
            check_arity("map:size", args, 1)?;
            let map = extract_map(&args[0], "map:size")?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Integer(map.len() as i64)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-map-keys
        "keys" => {
            check_arity("map:keys", args, 1)?;
            let map = extract_map(&args[0], "map:keys")?;
            let keys: XpathItemSet = map
                .iter()
                .map(|(k, _)| XpathItem::AnyAtomicType(k.clone()))
                .collect();
            Ok(Some(keys))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-map-contains
        "contains" => {
            check_arity("map:contains", args, 2)?;
            let map = extract_map(&args[0], "map:contains")?;
            if args[1].is_empty() {
                return Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                    AnyAtomicType::Boolean(false)
                )]));
            }
            let key = match &args[1][0] {
                XpathItem::AnyAtomicType(a) => a,
                _ => {
                    return Err(ExpressionApplyError::new(
                        "map:contains: key must be an atomic value".to_string(),
                    ));
                }
            };
            let found = map.iter().any(|(k, _)| k == key);
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Boolean(found)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-map-get
        "get" => {
            check_arity("map:get", args, 2)?;
            let map = extract_map(&args[0], "map:get")?;
            if args[1].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            let key = match &args[1][0] {
                XpathItem::AnyAtomicType(a) => a,
                _ => {
                    return Err(ExpressionApplyError::new(
                        "map:get: key must be an atomic value".to_string(),
                    ));
                }
            };
            for (k, v) in map {
                if k == key {
                    let result: XpathItemSet = v
                        .iter()
                        .map(|a| XpathItem::AnyAtomicType(a.clone()))
                        .collect();
                    return Ok(Some(result));
                }
            }
            Ok(Some(XpathItemSet::new()))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-map-put
        "put" => {
            check_arity("map:put", args, 3)?;
            let map = extract_map(&args[0], "map:put")?;
            if args[1].is_empty() {
                return Err(ExpressionApplyError::new(
                    "map:put: key must not be empty".to_string(),
                ));
            }
            let new_key = match &args[1][0] {
                XpathItem::AnyAtomicType(a) => a.clone(),
                _ => {
                    return Err(ExpressionApplyError::new(
                        "map:put: key must be an atomic value".to_string(),
                    ));
                }
            };
            let new_val: Vec<AnyAtomicType> = func_data(&args[2], context.item_tree)?;
            let mut new_entries: Vec<(AnyAtomicType, Vec<AnyAtomicType>)> = map
                .iter()
                .filter(|(k, _)| *k != new_key)
                .cloned()
                .collect();
            new_entries.push((new_key, new_val));
            Ok(Some(xpath_item_set![XpathItem::Function(Function::Map {
                entries: new_entries
            })]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-map-entry
        "entry" => {
            check_arity("map:entry", args, 2)?;
            if args[0].is_empty() {
                return Err(ExpressionApplyError::new(
                    "map:entry: key must not be empty".to_string(),
                ));
            }
            let key = match &args[0][0] {
                XpathItem::AnyAtomicType(a) => a.clone(),
                _ => {
                    return Err(ExpressionApplyError::new(
                        "map:entry: key must be an atomic value".to_string(),
                    ));
                }
            };
            let val: Vec<AnyAtomicType> = func_data(&args[1], context.item_tree)?;
            Ok(Some(xpath_item_set![XpathItem::Function(Function::Map {
                entries: vec![(key, val)]
            })]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-map-remove
        "remove" => {
            check_arity("map:remove", args, 2)?;
            let map = extract_map(&args[0], "map:remove")?;
            let keys_to_remove: Vec<&AnyAtomicType> = args[1]
                .iter()
                .filter_map(|item| match item {
                    XpathItem::AnyAtomicType(a) => Some(a),
                    _ => None,
                })
                .collect();
            let new_entries: Vec<(AnyAtomicType, Vec<AnyAtomicType>)> = map
                .iter()
                .filter(|(k, _)| !keys_to_remove.contains(&k))
                .cloned()
                .collect();
            Ok(Some(xpath_item_set![XpathItem::Function(Function::Map {
                entries: new_entries
            })]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-map-merge
        "merge" => {
            if args.is_empty() || args.len() > 2 {
                return Err(ExpressionApplyError::new(format!(
                    "map:merge expects 1-2 arguments, got {}",
                    args.len()
                )));
            }
            // Arg 2 is an options map (ignored); default duplicates policy is "use-first".
            let mut merged: Vec<(AnyAtomicType, Vec<AnyAtomicType>)> = Vec::new();
            for item in args[0].iter() {
                if let XpathItem::Function(Function::Map { entries }) = item {
                    for (k, v) in entries {
                        // First-wins: only insert if key not already present.
                        if !merged.iter().any(|(mk, _)| mk == k) {
                            merged.push((k.clone(), v.clone()));
                        }
                    }
                }
            }
            Ok(Some(xpath_item_set![XpathItem::Function(Function::Map {
                entries: merged
            })]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-map-for-each
        "for-each" => {
            check_arity("map:for-each", args, 2)?;
            let map = extract_map(&args[0], "map:for-each")?;
            let func = extract_function_item(&args[1], "map:for-each")?;
            let mut result = XpathItemSet::new();
            for (k, v) in map {
                let key_set = xpath_item_set![XpathItem::AnyAtomicType(k.clone())];
                let val_set: XpathItemSet = v
                    .iter()
                    .map(|a| XpathItem::AnyAtomicType(a.clone()))
                    .collect();
                let call_result =
                    invoke_function_item(func, vec![key_set, val_set], context)?;
                for r in call_result.into_iter() {
                    result.insert(r);
                }
            }
            Ok(Some(result))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-map-find
        "find" => {
            check_arity("map:find", args, 2)?;
            if args[1].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            let search_key = match &args[1][0] {
                XpathItem::AnyAtomicType(a) => a,
                _ => {
                    return Err(ExpressionApplyError::new(
                        "map:find: key must be an atomic value".to_string(),
                    ));
                }
            };
            // Recursively search all maps in the input for matching keys.
            // Per spec, map:find returns a single array(*) containing all found values.
            let mut found_values: Vec<Vec<AnyAtomicType>> = Vec::new();
            func_map_find_recursive(&args[0], search_key, &mut found_values);
            Ok(Some(xpath_item_set![XpathItem::Function(Function::Array {
                members: found_values,
            })]))
        }
        _ => Ok(None),
    }
}

/// Dispatch `array:*` functions.
///
/// NOTE: When adding a new function here, also add it to [`is_known_array_function`]
/// so that `fn:function-lookup` can find it.
fn dispatch_array_function<'tree>(
    local_name: &str,
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
) -> Result<Option<XpathItemSet<'tree>>, ExpressionApplyError> {
    match local_name {
        // https://www.w3.org/TR/xpath-functions-31/#func-array-size
        "size" => {
            check_arity("array:size", args, 1)?;
            let arr = extract_array(&args[0], "array:size")?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Integer(arr.len() as i64)
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-get
        "get" => {
            check_arity("array:get", args, 2)?;
            let arr = extract_array(&args[0], "array:get")?;
            let idx = extract_index(&args[1], context.item_tree, "array:get")?;
            if idx < 1 || idx > arr.len() {
                return Err(ExpressionApplyError::new(format!(
                    "array:get: index {} out of bounds (array size {})",
                    idx,
                    arr.len()
                )));
            }
            let member: XpathItemSet = arr[idx - 1]
                .iter()
                .map(|a| XpathItem::AnyAtomicType(a.clone()))
                .collect();
            Ok(Some(member))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-put
        "put" => {
            check_arity("array:put", args, 3)?;
            let arr = extract_array(&args[0], "array:put")?;
            let idx = extract_index(&args[1], context.item_tree, "array:put")?;
            if idx < 1 || idx > arr.len() {
                return Err(ExpressionApplyError::new(format!(
                    "array:put: index {} out of bounds (array size {})",
                    idx,
                    arr.len()
                )));
            }
            let new_val: Vec<AnyAtomicType> = func_data(&args[2], context.item_tree)?;
            let mut new_members = arr.clone();
            new_members[idx - 1] = new_val;
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-append
        "append" => {
            check_arity("array:append", args, 2)?;
            let arr = extract_array(&args[0], "array:append")?;
            let new_val: Vec<AnyAtomicType> = func_data(&args[1], context.item_tree)?;
            let mut new_members = arr.clone();
            new_members.push(new_val);
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-subarray
        "subarray" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(ExpressionApplyError::new(format!(
                    "array:subarray expects 2 or 3 arguments, got {}",
                    args.len()
                )));
            }
            let arr = extract_array(&args[0], "array:subarray")?;
            let start = extract_index(&args[1], context.item_tree, "array:subarray")?;
            if start < 1 || start > arr.len() + 1 {
                return Err(ExpressionApplyError::new(format!(
                    "array:subarray: start {} out of bounds",
                    start
                )));
            }
            let length = if args.len() == 3 {
                extract_index(&args[2], context.item_tree, "array:subarray")?
            } else {
                arr.len() - start + 1
            };
            let end = (start - 1 + length).min(arr.len());
            let new_members = arr[start - 1..end].to_vec();
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-remove
        "remove" => {
            check_arity("array:remove", args, 2)?;
            let arr = extract_array(&args[0], "array:remove")?;
            let positions: Vec<usize> = args[1]
                .iter()
                .filter_map(|item| match item {
                    XpathItem::AnyAtomicType(AnyAtomicType::Integer(n)) => Some(*n as usize),
                    _ => None,
                })
                .collect();
            let new_members: Vec<Vec<AnyAtomicType>> = arr
                .iter()
                .enumerate()
                .filter(|(i, _)| !positions.contains(&(i + 1)))
                .map(|(_, m)| m.clone())
                .collect();
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-insert-before
        "insert-before" => {
            check_arity("array:insert-before", args, 3)?;
            let arr = extract_array(&args[0], "array:insert-before")?;
            let pos = extract_index(&args[1], context.item_tree, "array:insert-before")?;
            if pos < 1 || pos > arr.len() + 1 {
                return Err(ExpressionApplyError::new(format!(
                    "array:insert-before: position {} out of bounds",
                    pos
                )));
            }
            let new_val: Vec<AnyAtomicType> = func_data(&args[2], context.item_tree)?;
            let mut new_members = arr.clone();
            new_members.insert(pos - 1, new_val);
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-head
        "head" => {
            check_arity("array:head", args, 1)?;
            let arr = extract_array(&args[0], "array:head")?;
            if arr.is_empty() {
                return Err(ExpressionApplyError::new(
                    "array:head: array is empty".to_string(),
                ));
            }
            let member: XpathItemSet = arr[0]
                .iter()
                .map(|a| XpathItem::AnyAtomicType(a.clone()))
                .collect();
            Ok(Some(member))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-tail
        "tail" => {
            check_arity("array:tail", args, 1)?;
            let arr = extract_array(&args[0], "array:tail")?;
            if arr.is_empty() {
                return Err(ExpressionApplyError::new(
                    "array:tail: array is empty".to_string(),
                ));
            }
            let new_members = arr[1..].to_vec();
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-reverse
        "reverse" => {
            check_arity("array:reverse", args, 1)?;
            let arr = extract_array(&args[0], "array:reverse")?;
            let mut new_members = arr.clone();
            new_members.reverse();
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-join
        "join" => {
            check_arity("array:join", args, 1)?;
            let mut all_members: Vec<Vec<AnyAtomicType>> = Vec::new();
            for item in args[0].iter() {
                if let XpathItem::Function(Function::Array { members }) = item {
                    all_members.extend(members.iter().cloned());
                }
            }
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: all_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-flatten
        "flatten" => {
            check_arity("array:flatten", args, 1)?;
            let mut result = XpathItemSet::new();
            func_array_flatten(&args[0], &mut result);
            Ok(Some(result))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-for-each
        "for-each" => {
            check_arity("array:for-each", args, 2)?;
            let arr = extract_array(&args[0], "array:for-each")?;
            let func = extract_function_item(&args[1], "array:for-each")?;
            let mut new_members: Vec<Vec<AnyAtomicType>> = Vec::new();
            for member in arr {
                let member_set: XpathItemSet = member
                    .iter()
                    .map(|a| XpathItem::AnyAtomicType(a.clone()))
                    .collect();
                let call_result =
                    invoke_function_item(func, vec![member_set], context)?;
                let atoms = func_data(&call_result, context.item_tree)?;
                new_members.push(atoms);
            }
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-filter
        "filter" => {
            check_arity("array:filter", args, 2)?;
            let arr = extract_array(&args[0], "array:filter")?;
            let func = extract_function_item(&args[1], "array:filter")?;
            let mut new_members: Vec<Vec<AnyAtomicType>> = Vec::new();
            for member in arr {
                let member_set: XpathItemSet = member
                    .iter()
                    .map(|a| XpathItem::AnyAtomicType(a.clone()))
                    .collect();
                let call_result =
                    invoke_function_item(func, vec![member_set], context)?;
                if call_result.boolean()? {
                    new_members.push(member.clone());
                }
            }
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-fold-left
        "fold-left" => {
            check_arity("array:fold-left", args, 3)?;
            let arr = extract_array(&args[0], "array:fold-left")?;
            let func = extract_function_item(&args[2], "array:fold-left")?;
            let mut accumulator = args[1].clone();
            for member in arr {
                let member_set: XpathItemSet = member
                    .iter()
                    .map(|a| XpathItem::AnyAtomicType(a.clone()))
                    .collect();
                accumulator =
                    invoke_function_item(func, vec![accumulator, member_set], context)?;
            }
            Ok(Some(accumulator))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-fold-right
        "fold-right" => {
            check_arity("array:fold-right", args, 3)?;
            let arr = extract_array(&args[0], "array:fold-right")?;
            let func = extract_function_item(&args[2], "array:fold-right")?;
            let mut accumulator = args[1].clone();
            for member in arr.iter().rev() {
                let member_set: XpathItemSet = member
                    .iter()
                    .map(|a| XpathItem::AnyAtomicType(a.clone()))
                    .collect();
                accumulator =
                    invoke_function_item(func, vec![member_set, accumulator], context)?;
            }
            Ok(Some(accumulator))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-for-each-pair
        "for-each-pair" => {
            check_arity("array:for-each-pair", args, 3)?;
            let arr1 = extract_array(&args[0], "array:for-each-pair")?;
            let arr2 = extract_array(&args[1], "array:for-each-pair")?;
            let func = extract_function_item(&args[2], "array:for-each-pair")?;
            let mut new_members: Vec<Vec<AnyAtomicType>> = Vec::new();
            for (m1, m2) in arr1.iter().zip(arr2.iter()) {
                let set1: XpathItemSet = m1.iter().map(|a| XpathItem::AnyAtomicType(a.clone())).collect();
                let set2: XpathItemSet = m2.iter().map(|a| XpathItem::AnyAtomicType(a.clone())).collect();
                let call_result = invoke_function_item(func, vec![set1, set2], context)?;
                let atoms = func_data(&call_result, context.item_tree)?;
                new_members.push(atoms);
            }
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        // https://www.w3.org/TR/xpath-functions-31/#func-array-sort
        "sort" => {
            if args.is_empty() || args.len() > 3 {
                return Err(ExpressionApplyError::new(format!(
                    "array:sort expects 1-3 arguments, got {}",
                    args.len()
                )));
            }
            let arr = extract_array(&args[0], "array:sort")?;
            // Spec: array:sort($array, $collation?, $key?). Arg 2 is collation (ignored),
            // arg 3 is the key function.
            let key_func_arg = if args.len() == 3 {
                Some(&args[2])
            } else {
                None
            };
            let mut members_with_keys: Vec<(Vec<AnyAtomicType>, String)> = Vec::new();
            for member in arr {
                let key = if let Some(key_arg) = key_func_arg {
                    let func = extract_function_item(key_arg, "array:sort")?;
                    let member_set: XpathItemSet = member
                        .iter()
                        .map(|a| XpathItem::AnyAtomicType(a.clone()))
                        .collect();
                    let key_result = invoke_function_item(func, vec![member_set], context)?;
                    extract_string_arg(&key_result, context.item_tree)?
                } else {
                    member
                        .iter()
                        .map(|a| {
                            func_string(
                                &XpathItem::AnyAtomicType(a.clone()),
                                context.item_tree,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .join("")
                };
                members_with_keys.push((member.clone(), key));
            }
            members_with_keys.sort_by(|(_, ka), (_, kb)| ka.cmp(kb));
            let new_members: Vec<Vec<AnyAtomicType>> =
                members_with_keys.into_iter().map(|(m, _)| m).collect();
            Ok(Some(xpath_item_set![XpathItem::Function(
                Function::Array { members: new_members }
            )]))
        }
        _ => Ok(None),
    }
}

/// Dispatch `math:*` functions.
///
/// NOTE: When adding a new function here, also add it to [`is_known_math_function`]
/// so that `fn:function-lookup` can find it.
fn dispatch_math_function<'tree>(
    local_name: &str,
    args: &[XpathItemSet<'tree>],
    context: &XpathExpressionContext<'tree>,
) -> Result<Option<XpathItemSet<'tree>>, ExpressionApplyError> {
    match local_name {
        "pi" => {
            check_arity("math:pi", args, 0)?;
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Double(ordered_float::OrderedFloat(std::f64::consts::PI))
            )]))
        }
        "exp" | "exp10" | "log" | "log10" | "sqrt" | "sin" | "cos" | "tan" | "asin"
        | "acos" | "atan" => {
            check_arity(&format!("math:{}", local_name), args, 1)?;
            if args[0].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            let val = extract_double(&args[0], context.item_tree)?;
            let result = match local_name {
                "exp" => val.exp(),
                "exp10" => 10f64.powf(val),
                "log" => val.ln(),
                "log10" => val.log10(),
                "sqrt" => val.sqrt(),
                "sin" => val.sin(),
                "cos" => val.cos(),
                "tan" => val.tan(),
                "asin" => val.asin(),
                "acos" => val.acos(),
                "atan" => val.atan(),
                _ => unreachable!(),
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Double(ordered_float::OrderedFloat(result))
            )]))
        }
        "pow" | "atan2" => {
            check_arity(&format!("math:{}", local_name), args, 2)?;
            if args[0].is_empty() || args[1].is_empty() {
                return Ok(Some(XpathItemSet::new()));
            }
            let x = extract_double(&args[0], context.item_tree)?;
            let y = extract_double(&args[1], context.item_tree)?;
            let result = match local_name {
                "pow" => x.powf(y),
                "atan2" => x.atan2(y),
                _ => unreachable!(),
            };
            Ok(Some(xpath_item_set![XpathItem::AnyAtomicType(
                AnyAtomicType::Double(ordered_float::OrderedFloat(result))
            )]))
        }
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

    let haystack = extract_string_arg(arg1_set, context.item_tree)?;

    let arg2_set = &args[1];
    if arg2_set.len() > 1 {
        return Err(ExpressionApplyError {
            msg: format!(
                "contains: unexpected item set length {} for second argument",
                arg2_set.len()
            ),
        });
    }

    let needle = extract_string_arg(arg2_set, context.item_tree)?;

    Ok(xpath_item_set![XpathItem::AnyAtomicType(
        AnyAtomicType::Boolean(haystack.contains(&needle))
    )])
}

/// https://www.w3.org/TR/2017/REC-xpath-31-20170321/#dt-atomization
pub(crate) fn func_data<'tree>(
    set: &XpathItemSet<'tree>,
    item_tree: &'tree XpathItemTree,
) -> Result<Vec<AnyAtomicType>, ExpressionApplyError> {
    fn atomize<'tree>(
        item: &XpathItem,
        item_tree: &'tree XpathItemTree,
    ) -> Result<AnyAtomicType, ExpressionApplyError> {
        match item {
            XpathItem::Node(node) => match node {
                XpathItemTreeNode::DocumentNode(_) => {
                    Ok(AnyAtomicType::String(node.text_content(item_tree)))
                }
                XpathItemTreeNode::ElementNode(_) => {
                    Ok(AnyAtomicType::String(node.text_content(item_tree)))
                }
                XpathItemTreeNode::PINode(pi) => Ok(AnyAtomicType::String(pi.data.clone())),
                XpathItemTreeNode::CommentNode(c) => {
                    Ok(AnyAtomicType::String(c.content.clone()))
                }
                XpathItemTreeNode::TextNode(text) => {
                    Ok(AnyAtomicType::String(text.content.clone()))
                }
                &XpathItemTreeNode::AttributeNode(attribute) => {
                    Ok(AnyAtomicType::String(attribute.value.clone()))
                }
                XpathItemTreeNode::DoctypeNode(_) => Ok(AnyAtomicType::String(String::new())),
            },
            XpathItem::Function(_) => Err(ExpressionApplyError::new(
                "err:FOTY0013: fn:data is not defined for function items".to_string(),
            )),
            XpathItem::AnyAtomicType(atomic) => Ok(atomic.clone()),
        }
    }

    set.iter()
        .map(|item| atomize(item, item_tree))
        .collect::<Result<Vec<_>, _>>()
}

/// https://www.w3.org/TR/xpath-functions-31/#func-string
pub(crate) fn func_string<'tree>(
    item: &XpathItem,
    item_tree: &'tree XpathItemTree,
) -> Result<String, ExpressionApplyError> {
    match item {
        XpathItem::Node(node) => match node {
            XpathItemTreeNode::DocumentNode(_) => Ok(node.text_content(item_tree)),
            XpathItemTreeNode::ElementNode(_) => Ok(node.text_content(item_tree)),
            XpathItemTreeNode::PINode(pi) => Ok(pi.data.clone()),
            XpathItemTreeNode::CommentNode(c) => Ok(c.content.clone()),
            XpathItemTreeNode::TextNode(text) => Ok(text.content.clone()),
            XpathItemTreeNode::AttributeNode(attribute) => Ok(attribute.value.clone()),
            XpathItemTreeNode::DoctypeNode(_) => Ok(String::new()),
        },
        XpathItem::AnyAtomicType(atomic) => match atomic {
            AnyAtomicType::Boolean(b) => Ok(b.to_string()),
            AnyAtomicType::Integer(n) => Ok(n.to_string()),
            AnyAtomicType::Float(n) => Ok(n.to_string()),
            AnyAtomicType::Double(n) => Ok(n.to_string()),
            AnyAtomicType::String(s) => Ok(s.clone()),
            AnyAtomicType::QName { .. } => Ok(atomic.to_string()),
        },
        XpathItem::Function(_) => Err(ExpressionApplyError::new(
            "err:FOTY0014: fn:string is not defined for function items".to_string(),
        )),
    }
}

/// Extract a string value from an argument set, returning empty string if the set is empty.
fn extract_string_arg(
    arg: &XpathItemSet,
    tree: &XpathItemTree,
) -> Result<String, ExpressionApplyError> {
    if arg.is_empty() {
        Ok(String::new())
    } else {
        func_string(&arg[0], tree)
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

fn check_arity_range(
    name: &str,
    args: &[XpathItemSet],
    min: usize,
    max: usize,
) -> Result<(), ExpressionApplyError> {
    if args.len() < min || args.len() > max {
        Err(ExpressionApplyError::new(format!(
            "{} expects {}-{} argument(s), got {}",
            name,
            min,
            max,
            args.len()
        )))
    } else {
        Ok(())
    }
}

/// Extract a single numeric value as a non-negative integer from an argument.
///
/// Validates that the value is finite, not NaN, non-negative, and a whole number.
/// Callers that require 1-based indexing should check `>= 1` separately.
fn extract_index(arg: &XpathItemSet, item_tree: &XpathItemTree, func_name: &str) -> Result<usize, ExpressionApplyError> {
    let val = extract_double(arg, item_tree)?;
    if val.is_nan() || val.is_infinite() || val < 0.0 || val != val.trunc() {
        return Err(ExpressionApplyError::new(format!(
            "{}: invalid index value {}",
            func_name, val
        )));
    }
    Ok(val as usize)
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
            let s = func_string(other, item_tree)?;
            s.trim().parse::<f64>().map_err(|_| {
                ExpressionApplyError::new(format!("Cannot convert '{}' to a number", s))
            })
        }
    }
}

/// Format a positive integer as bijective base-26 alphabetic (1=a, 26=z, 27=aa, 53=ba, etc.).
fn format_alpha(mut n: i64, base: u8) -> String {
    let mut result = Vec::new();
    while n > 0 {
        n -= 1;
        result.push((base + (n % 26) as u8) as char);
        n /= 26;
    }
    result.reverse();
    result.into_iter().collect()
}

/// Apply a unary numeric operation, preserving the source type.
/// Round an integer to the given precision using integer arithmetic,
/// avoiding the precision loss of i64→f64→i64 round-trip for large values.
fn round_integer(n: i64, precision: i32) -> i64 {
    if precision >= 0 {
        // Rounding at decimal places that don't exist for integers.
        return n;
    }
    let shift = (-precision) as u32;
    if shift >= 19 {
        // 10^19 > i64::MAX, so any integer rounds to 0.
        return 0;
    }
    let divisor = 10_i64.pow(shift);
    let remainder = match n.checked_rem(divisor) {
        Some(r) => r,
        None => return 0, // overflow (e.g. i64::MIN % -1)
    };
    let abs_remainder = remainder.unsigned_abs();
    let half = divisor.unsigned_abs() / 2;
    let truncated = match n.checked_sub(remainder) {
        Some(t) => t,
        None => return 0, // overflow near i64::MIN
    };
    if n >= 0 {
        if abs_remainder >= half {
            truncated.checked_add(divisor).unwrap_or(truncated)
        } else {
            truncated
        }
    } else if abs_remainder >= half {
        truncated.checked_sub(divisor).unwrap_or(truncated)
    } else {
        truncated
    }
}

/// Round-half-to-even for integers, using integer arithmetic.
fn round_half_to_even_integer(n: i64, precision: i32) -> i64 {
    if precision >= 0 {
        return n;
    }
    let shift = (-precision) as u32;
    if shift >= 19 {
        return 0;
    }
    let divisor = 10_i64.pow(shift);
    let remainder = match n.checked_rem(divisor) {
        Some(r) => r,
        None => return 0, // overflow (e.g. i64::MIN % -1)
    };
    let abs_remainder = remainder.unsigned_abs();
    let half = divisor.unsigned_abs() / 2;
    let truncated = match n.checked_sub(remainder) {
        Some(t) => t,
        None => return 0, // overflow near i64::MIN
    };
    match abs_remainder.cmp(&half) {
        std::cmp::Ordering::Less => truncated,
        std::cmp::Ordering::Greater => {
            if n >= 0 {
                truncated.checked_add(divisor).unwrap_or(truncated)
            } else {
                truncated.checked_sub(divisor).unwrap_or(truncated)
            }
        }
        std::cmp::Ordering::Equal => {
            // Tie: round to even (the quotient that is even).
            let quotient = truncated / divisor;
            if quotient % 2 == 0 {
                truncated
            } else if n >= 0 {
                truncated.checked_add(divisor).unwrap_or(truncated)
            } else {
                truncated.checked_sub(divisor).unwrap_or(truncated)
            }
        }
    }
}

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
    let atoms = func_data(&args[0], context.item_tree)?;
    let mut total_f64: f64 = 0.0;
    let mut total_i64: i64 = 0;
    let mut all_integers = true;
    for atom in &atoms {
        match atom {
            AnyAtomicType::Integer(n) => {
                if all_integers {
                    match total_i64.checked_add(*n) {
                        Some(v) => total_i64 = v,
                        None => all_integers = false,
                    }
                }
                total_f64 += *n as f64;
            }
            AnyAtomicType::Float(f) => {
                total_f64 += f.0 as f64;
                all_integers = false;
            }
            AnyAtomicType::Double(d) => {
                total_f64 += d.0;
                all_integers = false;
            }
            AnyAtomicType::String(s) => {
                // xs:untypedAtomic → cast to xs:double per XPath 3.1 F&O §15.4.5
                let d: f64 = s.trim().parse().map_err(|_| {
                    ExpressionApplyError::new(format!(
                        "fn:sum: cannot cast {:?} to xs:double",
                        s
                    ))
                })?;
                total_f64 += d;
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
            AnyAtomicType::Integer(total_i64)
        )])
    } else {
        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Double(ordered_float::OrderedFloat(total_f64))
        )])
    }
}

/// Convert an XPath replacement string to the syntax expected by the `regex` crate.
///
/// XPath uses `\1`, `\2`, etc. for backreferences and `\\` for a literal backslash.
/// The `regex` crate uses `$1`, `$2`, etc. and `$$` for a literal `$`.
fn xpath_replacement_to_regex(replacement: &str) -> String {
    let mut result = String::with_capacity(replacement.len());
    let chars: Vec<char> = replacement.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' {
            // Escape literal $ as $$ for the regex crate.
            result.push_str("$$");
            i += 1;
        } else if chars[i] == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next.is_ascii_digit() {
                // Convert \N backreference to ${N} for the regex crate.
                // Using ${N} avoids ambiguity when followed by more characters.
                result.push_str("${");
                result.push(next);
                result.push('}');
                i += 2;
            } else if next == '\\' {
                // Literal backslash.
                result.push('\\');
                i += 2;
            } else {
                // Other escaped characters: pass through as-is.
                result.push(chars[i]);
                result.push(next);
                i += 2;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// Compare two atomic values with cross-type numeric equality.
///
/// Per the XPath spec, numeric values of different types (Integer, Float, Double)
/// should be considered equal if their numeric values are equal
/// (e.g., `Integer(1)` equals `Double(1.0)`).
fn atoms_equal(a: &AnyAtomicType, b: &AnyAtomicType) -> bool {
    match (a, b) {
        // Cross-type numeric comparison: promote both to f64.
        (AnyAtomicType::Integer(i), AnyAtomicType::Double(d))
        | (AnyAtomicType::Double(d), AnyAtomicType::Integer(i)) => !d.0.is_nan() && (*i as f64) == d.0,
        (AnyAtomicType::Integer(i), AnyAtomicType::Float(f))
        | (AnyAtomicType::Float(f), AnyAtomicType::Integer(i)) => !f.0.is_nan() && (*i as f64) == (f.0 as f64),
        (AnyAtomicType::Float(f), AnyAtomicType::Double(d))
        | (AnyAtomicType::Double(d), AnyAtomicType::Float(f)) => !f.0.is_nan() && !d.0.is_nan() && (f.0 as f64) == d.0,
        // NaN is never equal to itself per IEEE 754 / XPath spec.
        (AnyAtomicType::Double(d1), AnyAtomicType::Double(d2)) => !d1.0.is_nan() && !d2.0.is_nan() && d1 == d2,
        (AnyAtomicType::Float(f1), AnyAtomicType::Float(f2)) => !f1.0.is_nan() && !f2.0.is_nan() && f1 == f2,
        // Same-type: use derived PartialEq.
        _ => a == b,
    }
}

/// Compare two atomized key sequences for sorting.
///
/// Uses `AnyAtomicType::partial_cmp` for same-type or numeric comparison,
/// falling back to string comparison for incomparable types.
fn sort_cmp_atomized(a: &[AnyAtomicType], b: &[AnyAtomicType]) -> std::cmp::Ordering {
    for (ai, bi) in a.iter().zip(b.iter()) {
        if let Some(ord) = ai.partial_cmp(bi) {
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        } else {
            // Incomparable types: fall back to string representation.
            let sa = ai.to_string();
            let sb = bi.to_string();
            let ord = sa.cmp(&sb);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
    }
    a.len().cmp(&b.len())
}

/// Build a `regex::Regex` from an XPath pattern string and flags.
///
/// Supported flags: `i` (case-insensitive), `s` (dot-all), `m` (multi-line), `x` (extended).
fn build_regex(pattern: &str, flags: &str) -> Result<regex::Regex, ExpressionApplyError> {
    let translated = translate_xpath_regex(pattern);
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
    regex_pattern.push_str(&translated);
    regex::Regex::new(&regex_pattern).map_err(|e| {
        ExpressionApplyError::new(format!("invalid regex pattern '{}': {}", pattern, e))
    })
}

/// Translate XPath-specific regex character class escapes to Rust regex equivalents.
///
/// XPath defines `\i`, `\I`, `\c`, `\C` which are not standard in Rust's regex crate.
/// Character class subtraction (`[X-[Y]]`) is a known limitation and not translated.
fn translate_xpath_regex(pattern: &str) -> String {
    let mut result = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    'i' => {
                        chars.next();
                        result.push_str("[\\p{L}_]");
                    }
                    'I' => {
                        chars.next();
                        result.push_str("[^\\p{L}_]");
                    }
                    'c' => {
                        chars.next();
                        result.push_str("[\\p{L}\\p{N}.\\-_:]");
                    }
                    'C' => {
                        chars.next();
                        result.push_str("[^\\p{L}\\p{N}.\\-_:]");
                    }
                    _ => {
                        result.push('\\');
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }
    result
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
    let func_name = if is_min { "min" } else { "max" };
    let atoms = func_data(&args[0], context.item_tree)?;

    // Determine comparison mode: all-numeric, all-string, or mixed (error).
    let has_numeric = atoms.iter().any(|a| {
        matches!(
            a,
            AnyAtomicType::Integer(_) | AnyAtomicType::Float(_) | AnyAtomicType::Double(_)
        )
    });
    let has_string = atoms.iter().any(|a| matches!(a, AnyAtomicType::String(_)));
    let has_other = atoms.iter().any(|a| {
        !matches!(
            a,
            AnyAtomicType::Integer(_)
                | AnyAtomicType::Float(_)
                | AnyAtomicType::Double(_)
                | AnyAtomicType::String(_)
        )
    });
    if has_other || (has_numeric && has_string) {
        return Err(ExpressionApplyError::new(format!(
            "err:FORG0006 fn:{func_name}: non-comparable value types"
        )));
    }

    if has_string {
        // All-string comparison.
        let mut best_str: Option<&str> = None;
        for atom in &atoms {
            if let AnyAtomicType::String(s) = atom {
                best_str = Some(match best_str {
                    None => s.as_str(),
                    Some(b) => {
                        if (is_min && s.as_str() < b) || (!is_min && s.as_str() > b) {
                            s.as_str()
                        } else {
                            b
                        }
                    }
                });
            }
        }
        return Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::String(best_str.unwrap().to_string())
        )]);
    }

    // Per XPath 3.1 spec: if any value is NaN, return NaN.
    let has_nan = atoms.iter().any(|a| match a {
        AnyAtomicType::Float(f) => f.0.is_nan(),
        AnyAtomicType::Double(d) => d.0.is_nan(),
        _ => false,
    });
    if has_nan {
        return Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Double(ordered_float::OrderedFloat(f64::NAN))
        )]);
    }

    // Numeric comparison with integer-precision preservation.
    let mut best_f64: f64 = if is_min { f64::INFINITY } else { f64::NEG_INFINITY };
    let mut best_i64: i64 = if is_min { i64::MAX } else { i64::MIN };
    let mut all_integers = true;
    for atom in &atoms {
        let val = match atom {
            AnyAtomicType::Integer(n) => {
                if all_integers {
                    if is_min {
                        if *n < best_i64 {
                            best_i64 = *n;
                        }
                    } else if *n > best_i64 {
                        best_i64 = *n;
                    }
                }
                *n as f64
            }
            AnyAtomicType::Float(f) => {
                all_integers = false;
                f.0 as f64
            }
            AnyAtomicType::Double(d) => {
                all_integers = false;
                d.0
            }
            other => {
                return Err(ExpressionApplyError::new(format!(
                    "fn:{func_name}: non-comparable value {:?}",
                    other
                )));
            }
        };
        if is_min {
            if val < best_f64 {
                best_f64 = val;
            }
        } else if val > best_f64 {
            best_f64 = val;
        }
    }
    if all_integers {
        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Integer(best_i64)
        )])
    } else {
        Ok(xpath_item_set![XpathItem::AnyAtomicType(
            AnyAtomicType::Double(ordered_float::OrderedFloat(best_f64))
        )])
    }
}

/// Convert a positive integer to a Roman numeral string. Returns None for values outside 1-3999.
fn func_to_roman(n: i64) -> Option<String> {
    if n < 1 || n > 3999 {
        return None;
    }
    let mut n = n as usize;
    let table = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    for &(value, numeral) in &table {
        while n >= value {
            result.push_str(numeral);
            n -= value;
        }
    }
    Some(result)
}

/// Convert an integer to English words (simplified). Falls back to digits for
/// values outside the supported range.
fn func_number_to_words(n: i64) -> String {
    if n == 0 {
        return "Zero".to_string();
    }
    let negative = n < 0;
    let n = n.unsigned_abs();
    // Fall back to digits for values too large for word representation.
    if n >= 20_000 {
        return if negative {
            format!("Minus {}", n)
        } else {
            n.to_string()
        };
    }
    let ones = [
        "", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten",
        "Eleven", "Twelve", "Thirteen", "Fourteen", "Fifteen", "Sixteen", "Seventeen",
        "Eighteen", "Nineteen",
    ];
    let tens = [
        "", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety",
    ];
    let mut parts = Vec::new();
    if negative {
        parts.push("Minus".to_string());
    }
    if n >= 1000 {
        let thousands = n / 1000;
        parts.push(format!("{} Thousand", ones[thousands as usize]));
        let remainder = n % 1000;
        if remainder > 0 {
            parts.push(func_number_to_words(remainder as i64));
        }
    } else if n >= 100 {
        let hundreds = n / 100;
        parts.push(format!("{} Hundred", ones[hundreds as usize]));
        let remainder = n % 100;
        if remainder > 0 {
            parts.push(func_number_to_words(remainder as i64));
        }
    } else if n >= 20 {
        let t = n / 10;
        let o = n % 10;
        if o > 0 {
            parts.push(format!("{} {}", tens[t as usize], ones[o as usize]));
        } else {
            parts.push(tens[t as usize].to_string());
        }
    } else {
        parts.push(ones[n as usize].to_string());
    }
    parts.join(" ")
}

/// Build the XPath path expression for a node (e.g., `/document-node()/html[1]/body[1]`).
fn func_node_path(node: &XpathItemTreeNode, tree: &XpathItemTree) -> String {
    match node {
        XpathItemTreeNode::DocumentNode(_) => "/".to_string(),
        _ => {
            // Walk up to root, collecting path segments.
            let mut segments = Vec::new();
            let mut current = Some(node);
            while let Some(cur) = current {
                match cur {
                    XpathItemTreeNode::DocumentNode(_) => break,
                    XpathItemTreeNode::ElementNode(e) => {
                        // Count preceding siblings with the same name for the positional predicate.
                        let pos = if let Some(nid) = cur.node_id() {
                            let mut count = 1;
                            let mut prev = tree.arena.get(nid).and_then(|n| n.previous_sibling());
                            while let Some(sib_id) = prev {
                                if let XpathItemTreeNode::ElementNode(se) = tree.get(sib_id) {
                                    if se.name == e.name {
                                        count += 1;
                                    }
                                }
                                prev = tree.arena.get(sib_id).and_then(|n| n.previous_sibling());
                            }
                            count
                        } else {
                            1
                        };
                        segments.push(format!("{}[{}]", e.name, pos));
                    }
                    XpathItemTreeNode::AttributeNode(a) => {
                        segments.push(format!("@{}", a.name));
                    }
                    XpathItemTreeNode::TextNode(_) => {
                        // Count preceding text siblings.
                        let pos = if let Some(nid) = cur.node_id() {
                            let mut count = 1;
                            let mut prev = tree.arena.get(nid).and_then(|n| n.previous_sibling());
                            while let Some(sib_id) = prev {
                                if matches!(tree.get(sib_id), XpathItemTreeNode::TextNode(_)) {
                                    count += 1;
                                }
                                prev = tree.arena.get(sib_id).and_then(|n| n.previous_sibling());
                            }
                            count
                        } else {
                            1
                        };
                        segments.push(format!("text()[{}]", pos));
                    }
                    XpathItemTreeNode::CommentNode(_) => {
                        let pos = if let Some(nid) = cur.node_id() {
                            let mut count = 1;
                            let mut prev = tree.arena.get(nid).and_then(|n| n.previous_sibling());
                            while let Some(sib_id) = prev {
                                if matches!(tree.get(sib_id), XpathItemTreeNode::CommentNode(_)) {
                                    count += 1;
                                }
                                prev = tree.arena.get(sib_id).and_then(|n| n.previous_sibling());
                            }
                            count
                        } else {
                            1
                        };
                        segments.push(format!("comment()[{}]", pos));
                    }
                    _ => segments.push("?".to_string()),
                }
                current = cur.parent(tree);
            }
            segments.reverse();
            format!("/{}", segments.join("/"))
        }
    }
}

/// Extract a function item from an XpathItemSet, returning an error if it's not a single function.
fn extract_function_item<'a, 'tree>(
    items: &'a XpathItemSet<'tree>,
    fn_name: &str,
) -> Result<&'a Function, ExpressionApplyError> {
    if items.len() != 1 {
        return Err(ExpressionApplyError::new(format!(
            "{}: function argument must be a single function item, got {} items",
            fn_name,
            items.len()
        )));
    }
    match &items[0] {
        XpathItem::Function(f) => Ok(f),
        _ => Err(ExpressionApplyError::new(format!(
            "{}: argument is not a function item",
            fn_name
        ))),
    }
}

/// Extract a map from the first item of an XpathItemSet.
fn extract_map<'a>(
    items: &'a XpathItemSet,
    fn_name: &str,
) -> Result<&'a Vec<(AnyAtomicType, Vec<AnyAtomicType>)>, ExpressionApplyError> {
    if items.is_empty() {
        return Err(ExpressionApplyError::new(format!(
            "{}: argument is empty",
            fn_name
        )));
    }
    match &items[0] {
        XpathItem::Function(Function::Map { entries }) => Ok(entries),
        _ => Err(ExpressionApplyError::new(format!(
            "{}: argument is not a map",
            fn_name
        ))),
    }
}

/// Extract an array from the first item of an XpathItemSet.
fn extract_array<'a>(
    items: &'a XpathItemSet,
    fn_name: &str,
) -> Result<&'a Vec<Vec<AnyAtomicType>>, ExpressionApplyError> {
    if items.is_empty() {
        return Err(ExpressionApplyError::new(format!(
            "{}: argument is empty",
            fn_name
        )));
    }
    match &items[0] {
        XpathItem::Function(Function::Array { members }) => Ok(members),
        _ => Err(ExpressionApplyError::new(format!(
            "{}: argument is not an array",
            fn_name
        ))),
    }
}

/// Recursively search for map entries with a given key.
fn func_map_find_recursive(
    items: &XpathItemSet<'_>,
    key: &AnyAtomicType,
    found_values: &mut Vec<Vec<AnyAtomicType>>,
) {
    for item in items.iter() {
        match item {
            XpathItem::Function(Function::Map { entries }) => {
                for (k, v) in entries {
                    if k == key {
                        found_values.push(v.clone());
                    }
                    // Recurse into nested map values.
                    let nested: XpathItemSet = v
                        .iter()
                        .map(|a| XpathItem::AnyAtomicType(a.clone()))
                        .collect();
                    func_map_find_recursive(&nested, key, found_values);
                }
            }
            XpathItem::Function(Function::Array { members }) => {
                for member in members {
                    let nested: XpathItemSet = member
                        .iter()
                        .map(|a| XpathItem::AnyAtomicType(a.clone()))
                        .collect();
                    func_map_find_recursive(&nested, key, found_values);
                }
            }
            _ => {}
        }
    }
}

/// Recursively flatten arrays into a sequence of atomic values.
fn func_array_flatten<'tree>(items: &XpathItemSet<'tree>, result: &mut XpathItemSet<'tree>) {
    for item in items.iter() {
        match item {
            XpathItem::Function(Function::Array { members }) => {
                for member in members {
                    let nested: XpathItemSet = member
                        .iter()
                        .map(|a| XpathItem::AnyAtomicType(a.clone()))
                        .collect();
                    func_array_flatten(&nested, result);
                }
            }
            other => {
                result.insert(other.clone());
            }
        }
    }
}

/// Format a number according to a picture string (subset of XPath 3.1 spec).
/// Supports: `#` (optional digit), `0`-`9` (mandatory digit positions), `.` (decimal separator),
/// `,` (grouping separator), `%` (percent), `;` (sub-picture separator for negatives).
fn func_format_number(value: f64, picture: &str) -> Result<String, ExpressionApplyError> {
    if value.is_nan() {
        return Ok("NaN".to_string());
    }
    if value.is_infinite() {
        return if value > 0.0 {
            Ok("Infinity".to_string())
        } else {
            Ok("-Infinity".to_string())
        };
    }

    fn is_digit_char(c: char) -> bool {
        // In XPath format-number, 0-9 are all mandatory digit positions, # is optional.
        c.is_ascii_digit() || c == '#'
    }
    fn is_active_char(c: char) -> bool {
        is_digit_char(c) || c == '.' || c == ','
    }

    // Split into positive/negative sub-pictures.
    let (pos_pic, neg_pic) = if let Some(idx) = picture.find(';') {
        (&picture[..idx], Some(&picture[idx + 1..]))
    } else {
        (picture.as_ref(), None)
    };

    let is_negative = value < 0.0;
    let sub_picture = if is_negative {
        neg_pic.unwrap_or(pos_pic)
    } else {
        pos_pic
    };

    // Check for percent.
    let has_percent = sub_picture.contains('%');
    let abs_value = if has_percent {
        value.abs() * 100.0
    } else {
        value.abs()
    };

    // Extract prefix and suffix (passive characters before/after the active part).
    let first_active = sub_picture.find(is_active_char).unwrap_or(sub_picture.len());
    let last_active = sub_picture.rfind(is_active_char).map(|i| i + 1).unwrap_or(0);
    let prefix = &sub_picture[..first_active];
    let suffix = &sub_picture[last_active..];
    let pattern = &sub_picture[first_active..last_active];

    // Parse the active pattern.
    let (int_pattern, frac_pattern) = if let Some(dot_pos) = pattern.find('.') {
        (&pattern[..dot_pos], Some(&pattern[dot_pos + 1..]))
    } else {
        (pattern, None)
    };

    // Determine fractional digits. 0-9 are mandatory, # is optional.
    let min_frac = frac_pattern.map_or(0, |p| {
        p.chars().filter(|c| c.is_ascii_digit()).count()
    });
    let max_frac = frac_pattern.map_or(0, |p| {
        p.chars().filter(|c| is_digit_char(*c)).count()
    });

    // Round to max fractional digits.
    let factor = 10f64.powi(max_frac as i32);
    let rounded = (abs_value * factor).round() / factor;

    // Split into integer and fractional string parts.
    let formatted = format!("{:.prec$}", rounded, prec = max_frac);
    let (int_str, frac_str) = if let Some(dot) = formatted.find('.') {
        (&formatted[..dot], Some(&formatted[dot + 1..]))
    } else {
        (formatted.as_str(), None)
    };

    // Determine minimum integer digits (count of 0-9 chars in integer pattern).
    let min_int = int_pattern.chars().filter(|c| c.is_ascii_digit()).count();
    let mut int_digits = int_str.to_string();
    while int_digits.len() < min_int.max(1) {
        int_digits.insert(0, '0');
    }

    // Apply grouping separators to integer part.
    // Determine regular grouping size from the rightmost comma.
    let group_size = if int_pattern.contains(',') {
        let after_last_comma =
            int_pattern.rfind(',').map(|i| &int_pattern[i + 1..]).unwrap_or("");
        after_last_comma.chars().filter(|c| is_digit_char(*c)).count()
    } else {
        0
    };

    if group_size > 0 {
        let mut grouped = String::new();
        for (i, ch) in int_digits.chars().rev().enumerate() {
            if i > 0 && i % group_size == 0 {
                grouped.push(',');
            }
            grouped.push(ch);
        }
        int_digits = grouped.chars().rev().collect();
    }

    // Build fractional part.
    let frac_result = if max_frac > 0 {
        let mut frac = frac_str.unwrap_or("").to_string();
        // Trim trailing zeros beyond min_frac.
        while frac.len() > min_frac && frac.ends_with('0') {
            frac.pop();
        }
        // Pad to min_frac.
        while frac.len() < min_frac {
            frac.push('0');
        }
        if frac.is_empty() {
            None
        } else {
            Some(frac)
        }
    } else {
        None
    };

    // Assemble the result.
    let mut result = String::new();
    if is_negative && neg_pic.is_none() {
        result.push('-');
    }
    result.push_str(prefix);
    result.push_str(&int_digits);
    if let Some(frac) = frac_result {
        result.push('.');
        result.push_str(&frac);
    }
    result.push_str(suffix);

    Ok(result)
}

/// Apply Unicode normalization to a string.
fn func_normalize_unicode(input: &str, form: &str) -> Result<String, ExpressionApplyError> {
    use unicode_normalization::UnicodeNormalization;

    if form.is_empty() {
        return Ok(input.to_string());
    }

    match form {
        "NFC" => Ok(input.nfc().collect()),
        "NFD" => Ok(input.nfd().collect()),
        "NFKC" => Ok(input.nfkc().collect()),
        "NFKD" => Ok(input.nfkd().collect()),
        _ => Err(ExpressionApplyError::new(format!(
            "fn:normalize-unicode: unsupported normalization form '{}'",
            form
        ))),
    }
}

/// Structurally compare two XPath items for deep equality per the XPath 3.1 spec.
/// <https://www.w3.org/TR/xpath-functions-31/#func-deep-equal>
fn deep_equal_items(
    a: &XpathItem,
    b: &XpathItem,
    tree: &XpathItemTree,
) -> Result<bool, ExpressionApplyError> {
    match (a, b) {
        // Atomic values: use cross-type numeric equality.
        (XpathItem::AnyAtomicType(a), XpathItem::AnyAtomicType(b)) => Ok(atoms_equal(a, b)),
        // Functions: per XPath 3.1, fn:deep-equal raises err:FOTY0015 for function items.
        (XpathItem::Function(_), _) | (_, XpathItem::Function(_)) => {
            Err(ExpressionApplyError::new(
                "err:FOTY0015 fn:deep-equal does not support function items".to_string(),
            ))
        }
        // Nodes: structural comparison.
        (XpathItem::Node(a), XpathItem::Node(b)) => Ok(deep_equal_nodes(a, b, tree)),
        // Different kinds of items are never deep-equal.
        _ => Ok(false),
    }
}

/// Structurally compare two XPath nodes for deep equality.
fn deep_equal_nodes(a: &XpathItemTreeNode, b: &XpathItemTreeNode, tree: &XpathItemTree) -> bool {
    use XpathItemTreeNode::*;
    match (a, b) {
        (DocumentNode(_), DocumentNode(_)) => {
            // Deep-equal on document nodes: compare child elements and text nodes.
            let a_children = content_children(a, tree);
            let b_children = content_children(b, tree);
            a_children.len() == b_children.len()
                && a_children
                    .iter()
                    .zip(b_children.iter())
                    .all(|(ac, bc)| deep_equal_nodes(ac, bc, tree))
        }
        (ElementNode(ae), ElementNode(be)) => {
            // Same name and namespace.
            if ae.name != be.name || ae.namespace != be.namespace {
                return false;
            }
            // Same attributes (order-insensitive).
            let a_attrs = ae.attributes(tree);
            let b_attrs = be.attributes(tree);
            if a_attrs.len() != b_attrs.len() {
                return false;
            }
            // For every attribute in a, there must be a matching attribute in b.
            for a_attr in &a_attrs {
                if !b_attrs
                    .iter()
                    .any(|b_attr| a_attr.name == b_attr.name && a_attr.value == b_attr.value)
                {
                    return false;
                }
            }
            // Compare child content (elements and text, not attributes).
            let a_children = content_children(a, tree);
            let b_children = content_children(b, tree);
            a_children.len() == b_children.len()
                && a_children
                    .iter()
                    .zip(b_children.iter())
                    .all(|(ac, bc)| deep_equal_nodes(ac, bc, tree))
        }
        (TextNode(at), TextNode(bt)) => at.content == bt.content,
        (CommentNode(ac), CommentNode(bc)) => ac.content == bc.content,
        (PINode(ap), PINode(bp)) => ap.target == bp.target && ap.data == bp.data,
        (AttributeNode(aa), AttributeNode(ba)) => aa.name == ba.name && aa.value == ba.value,
        // Different node kinds are never deep-equal.
        _ => false,
    }
}

/// Get the content children (elements and text nodes) of a node, excluding attributes.
fn content_children<'tree>(
    node: &'tree XpathItemTreeNode,
    tree: &'tree XpathItemTree,
) -> Vec<&'tree XpathItemTreeNode> {
    node.children(tree)
        .into_iter()
        .filter(|child| {
            matches!(
                child,
                XpathItemTreeNode::ElementNode(_) | XpathItemTreeNode::TextNode(_)
            )
        })
        .collect()
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
