//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-context-item-expression>

use std::fmt::Display;

use nom::error::context;

use crate::{
    xpath::{
        grammar::{
            data_model::{AnyAtomicType, XpathItem},
            expressions::common::{argument_list, ArgumentList},
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
        match &self.name {
            EQName::QName(qname) => match qname {
                QName::PrefixedName(prefixed_name) => {
                    if prefixed_name.prefix == "fn" {
                        // Root function selects the root node of the tree.
                        if prefixed_name.local_part == "root" {
                            return Ok(xpath_item_set![XpathItem::Node(context.item_tree.root())]);
                        }
                    }

                    Err(ExpressionApplyError {
                        msg: format!("Unknown function {}", self.name.to_string()),
                    })
                }
                QName::UnprefixedName(unprefixed_name) => match unprefixed_name.as_str() {
                    "contains" => func_contains(&self.argument_list, context),
                    _ => Err(ExpressionApplyError {
                        msg: format!("Unknown function {}", self.name.to_string()),
                    }),
                },
            },
            EQName::UriQualifiedName(_) => todo!("FunctionCall::eval UriQualifiedName"),
        }
    }
}

/// https://developer.mozilla.org/en-US/docs/Web/XPath/Functions/contains
fn func_contains<'tree>(
    argument_list: &ArgumentList,
    context: &XpathExpressionContext<'tree>,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    let arguments = &argument_list.0;
    if arguments.len() != 2 {
        return Err(ExpressionApplyError {
            msg: format!(
                "contains: function expects 2 arguments, got {}",
                arguments.len()
            ),
        });
    }

    let arg1_set = arguments[0].eval(context)?;
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

    let arg2_set = arguments[1].eval(context)?;
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
                XpathItemTreeNode::PINode(_) => todo!("func_data PINode"),
                XpathItemTreeNode::CommentNode(_) => todo!("func_data CommentNode"),
                XpathItemTreeNode::TextNode(text) => AnyAtomicType::String(text.content.clone()),
                &XpathItemTreeNode::AttributeNode(attribute) => {
                    AnyAtomicType::String(attribute.value.clone())
                }
            },
            XpathItem::Function(_) => todo!("func_data Function"),
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
            XpathItemTreeNode::PINode(_) => todo!("func_string PINode"),
            XpathItemTreeNode::CommentNode(_) => todo!("func_string CommentNode"),
            XpathItemTreeNode::TextNode(text) => text.content.clone(),
            XpathItemTreeNode::AttributeNode(attribute) => attribute.value.clone(),
        },
        XpathItem::AnyAtomicType(atomic) => match atomic {
            AnyAtomicType::Boolean(b) => b.to_string(),
            AnyAtomicType::Integer(n) => n.to_string(),
            AnyAtomicType::Float(n) => n.to_string(),
            AnyAtomicType::Double(n) => n.to_string(),
            AnyAtomicType::String(s) => s.clone(),
        },
        XpathItem::Function(_) => todo!("func_string Function"),
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
