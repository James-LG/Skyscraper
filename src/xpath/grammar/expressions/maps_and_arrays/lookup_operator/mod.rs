//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-lookup>

use crate::xpath::{
    grammar::data_model::{AnyAtomicType, Function, OwnedXpathValue, XpathItem},
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use self::unary_lookup::KeySpecifier;

pub mod postfix_lookup;
pub mod unary_lookup;

/// Perform a lookup on a single function item (map or array) using a key specifier.
///
/// For maps:
/// - `Name(s)` → look up string key `s`
/// - `Integer(n)` → look up integer key `n`
/// - `Wildcard` → return all values
/// - `ParenthesizedExpr(e)` → evaluate `e` and use result as key
///
/// For arrays:
/// - `Integer(n)` → return member at 1-based index `n`
/// - `Wildcard` → return all members
/// - `ParenthesizedExpr(e)` → evaluate `e`, must be integer, use as index
/// - `Name(s)` → error (arrays don't support named keys)
pub(crate) fn apply_key_specifier<'tree>(
    func: &Function,
    key_spec: &KeySpecifier,
    context: &XpathExpressionContext<'tree>,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    match func {
        Function::Map { entries } => apply_key_to_map(entries, key_spec, context),
        Function::Array { members } => apply_key_to_array(members, key_spec, context),
        other => Err(ExpressionApplyError::new(format!(
            "Lookup operator requires a map or array, got {:?}",
            other
        ))),
    }
}

fn apply_key_to_map<'tree>(
    entries: &indexmap::IndexMap<AnyAtomicType, Vec<OwnedXpathValue>>,
    key_spec: &KeySpecifier,
    context: &XpathExpressionContext<'tree>,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    match key_spec {
        KeySpecifier::Name(name) => {
            let key = AnyAtomicType::String(name.clone());
            lookup_map_key(entries, &key)
        }
        KeySpecifier::Integer(n) => {
            let key = AnyAtomicType::Integer(*n as i64);
            lookup_map_key(entries, &key)
        }
        KeySpecifier::Wildcard => {
            // Return all values from all entries.
            let mut result = XpathItemSet::new();
            for (_, values) in entries {
                for v in values {
                    result.insert(v.to_xpath_item());
                }
            }
            Ok(result)
        }
        KeySpecifier::ParenthesizedExpr(expr) => {
            let key_set = expr.eval(context)?;
            if key_set.len() != 1 {
                return Err(ExpressionApplyError::new(format!(
                    "Lookup key expression must produce a single item, got {}",
                    key_set.len()
                )));
            }
            let key = match &key_set[0] {
                XpathItem::AnyAtomicType(a) => a.clone(),
                other => {
                    return Err(ExpressionApplyError::new(format!(
                        "Lookup key must be an atomic value, got {:?}",
                        other
                    )));
                }
            };
            lookup_map_key(entries, &key)
        }
    }
}

fn lookup_map_key<'tree>(
    entries: &indexmap::IndexMap<AnyAtomicType, Vec<OwnedXpathValue>>,
    key: &AnyAtomicType,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    if let Some(v) = entries.get(key) {
        Ok(v.iter().map(|a| a.to_xpath_item()).collect())
    } else {
        Ok(XpathItemSet::new())
    }
}

fn apply_key_to_array<'tree>(
    members: &[Vec<OwnedXpathValue>],
    key_spec: &KeySpecifier,
    context: &XpathExpressionContext<'tree>,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    match key_spec {
        KeySpecifier::Integer(n) => lookup_array_index(members, *n as i64),
        KeySpecifier::Wildcard => {
            // Return all members flattened.
            let mut result = XpathItemSet::new();
            for member in members {
                for v in member {
                    result.insert(v.to_xpath_item());
                }
            }
            Ok(result)
        }
        KeySpecifier::ParenthesizedExpr(expr) => {
            let idx_set = expr.eval(context)?;
            if idx_set.len() != 1 {
                return Err(ExpressionApplyError::new(format!(
                    "Array lookup index expression must produce a single item, got {}",
                    idx_set.len()
                )));
            }
            let idx = match &idx_set[0] {
                XpathItem::AnyAtomicType(AnyAtomicType::Integer(n)) => *n,
                other => {
                    return Err(ExpressionApplyError::new(format!(
                        "Array lookup index must be an integer, got {:?}",
                        other
                    )));
                }
            };
            lookup_array_index(members, idx)
        }
        KeySpecifier::Name(name) => Err(ExpressionApplyError::new(format!(
            "Array does not support named key lookup: {}",
            name
        ))),
    }
}

/// Call a map or array as a function with a single atomic argument.
///
/// This is the shared implementation for `$map("key")` and `$array(N)` dynamic
/// function calls. For maps, the argument is used as the lookup key. For arrays,
/// the argument must be an integer used as a 1-based index.
pub(crate) fn call_with_key<'tree>(
    func: &Function,
    key: &AnyAtomicType,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    match func {
        Function::Map { entries } => lookup_map_key(entries, key),
        Function::Array { members } => {
            let idx = match key {
                AnyAtomicType::Integer(n) => *n,
                other => {
                    return Err(ExpressionApplyError::new(format!(
                        "Array function call argument must be an integer, got {:?}",
                        other
                    )));
                }
            };
            lookup_array_index(members, idx)
        }
        other => Err(ExpressionApplyError::new(format!(
            "Cannot call {:?} as a map or array function",
            other
        ))),
    }
}

fn lookup_array_index<'tree>(
    members: &[Vec<OwnedXpathValue>],
    idx: i64,
) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
    if idx < 1 || idx as usize > members.len() {
        return Err(ExpressionApplyError::new(format!(
            "Array index {} out of bounds (array size: {})",
            idx,
            members.len()
        )));
    }
    let member = &members[(idx - 1) as usize];
    Ok(member
        .iter()
        .map(|a| a.to_xpath_item())
        .collect())
}
