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
