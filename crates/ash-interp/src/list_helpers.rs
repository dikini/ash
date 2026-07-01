//! List helpers for working with Cons/Nil variant representations
//!
//! As part of Phase 153, the legacy list runtime variant has been removed; lists use Cons/Nil.
//! Lists are now represented as nested Cons/Nil variants:
//!   []        = Value::Variant { name: "Nil", fields: [] }
//!   [a, b, c] = Value::Variant { name: "Cons", fields: [("head", a), ("tail", Cons ...)] }
//!
//! This module provides helper functions to ease the transition.

use ash_core::Value;

/// Create a Nil (empty list) value
pub fn nil() -> Value {
    Value::Variant {
        name: "Nil".to_string(),
        fields: Box::new(vec![]),
    }
}

/// Create a Cons (non-empty list) value
pub fn cons(head: Value, tail: Value) -> Value {
    Value::Variant {
        name: "Cons".to_string(),
        fields: Box::new(vec![("head".to_string(), head), ("tail".to_string(), tail)]),
    }
}

/// Check if a value is a list (Nil or Cons variant)
pub fn is_list(value: &Value) -> bool {
    match value {
        Value::Variant { name, .. } => name == "Nil" || name == "Cons",
        _ => false,
    }
}

/// Check if a value is Nil (empty list)
pub fn is_nil(value: &Value) -> bool {
    match value {
        Value::Variant { name, fields } => name == "Nil" && fields.is_empty(),
        _ => false,
    }
}

/// Get the head of a Cons list
pub fn list_head(value: &Value) -> Option<&Value> {
    match value {
        Value::Variant { name, fields } if name == "Cons" => {
            fields.iter().find(|(n, _)| n == "head").map(|(_, v)| v)
        }
        _ => None,
    }
}

/// Get the tail of a Cons list
pub fn list_tail(value: &Value) -> Option<&Value> {
    match value {
        Value::Variant { name, fields } if name == "Cons" => {
            fields.iter().find(|(n, _)| n == "tail").map(|(_, v)| v)
        }
        _ => None,
    }
}

/// Convert a `Vec<Value>` to a list (nested Cons/Nil)
pub fn vec_to_list(values: Vec<Value>) -> Value {
    let mut result = nil();
    for value in values.into_iter().rev() {
        result = cons(value, result);
    }
    result
}

/// Convert a list (nested Cons/Nil) to a `Vec<Value>`
pub fn list_to_vec(value: &Value) -> Option<Vec<Value>> {
    let mut result = Vec::new();
    let mut current = value;
    loop {
        match current {
            Value::Variant { name, fields } if name == "Nil" => {
                return Some(result);
            }
            Value::Variant { name, fields } if name == "Cons" => {
                let head = fields.iter().find(|(n, _)| n == "head").map(|(_, v)| v)?;
                let tail = fields.iter().find(|(n, _)| n == "tail").map(|(_, v)| v)?;
                result.push(head.clone());
                current = tail;
            }
            _ => return None,
        }
    }
}

/// Get the length of a list
pub fn list_len(value: &Value) -> Option<usize> {
    let mut count = 0;
    let mut current = value;
    loop {
        match current {
            Value::Variant { name, fields } if name == "Nil" => {
                return Some(count);
            }
            Value::Variant { name, fields } if name == "Cons" => {
                let tail = fields.iter().find(|(n, _)| n == "tail").map(|(_, v)| v)?;
                count += 1;
                current = tail;
            }
            _ => return None,
        }
    }
}

/// Get the nth element of a list
pub fn list_nth(value: &Value, n: usize) -> Option<&Value> {
    let mut current = value;
    let mut remaining = n;
    loop {
        match current {
            Value::Variant { name, fields } if name == "Nil" => {
                return None;
            }
            Value::Variant { name, fields } if name == "Cons" => {
                let head = fields.iter().find(|(n, _)| n == "head").map(|(_, v)| v)?;
                let tail = fields.iter().find(|(n, _)| n == "tail").map(|(_, v)| v)?;
                if remaining == 0 {
                    return Some(head);
                }
                remaining -= 1;
                current = tail;
            }
            _ => return None,
        }
    }
}

/// Append an element to a list (returns a new list)
pub fn list_append(list: &Value, item: Value) -> Option<Value> {
    let mut elements = Vec::new();
    let mut current = list;
    // First, collect all elements
    loop {
        match current {
            Value::Variant { name, fields } if name == "Nil" => {
                break;
            }
            Value::Variant { name, fields } if name == "Cons" => {
                let head = fields.iter().find(|(n, _)| n == "head").map(|(_, v)| v)?;
                let tail = fields.iter().find(|(n, _)| n == "tail").map(|(_, v)| v)?;
                elements.push(head.clone());
                current = tail;
            }
            _ => return None,
        }
    }
    // Build new list with appended element
    elements.push(item);
    Some(vec_to_list(elements))
}

/// Concatenate two lists
pub fn list_concat(left: &Value, right: &Value) -> Option<Value> {
    let left_elements = list_to_vec(left)?;
    let right_elements = list_to_vec(right)?;
    let mut result = left_elements;
    result.extend(right_elements);
    Some(vec_to_list(result))
}

/// Take n elements from a list
pub fn list_take(list: &Value, n: usize) -> Option<Value> {
    let elements = list_to_vec(list)?;
    let take_count = n.min(elements.len());
    Some(vec_to_list(elements.into_iter().take(take_count).collect()))
}

/// Drop n elements from a list
pub fn list_drop(list: &Value, n: usize) -> Option<Value> {
    let elements = list_to_vec(list)?;
    let drop_count = n.min(elements.len());
    Some(vec_to_list(elements.into_iter().skip(drop_count).collect()))
}

/// Reverse a list
pub fn list_reverse(list: &Value) -> Option<Value> {
    let mut elements = list_to_vec(list)?;
    elements.reverse();
    Some(vec_to_list(elements))
}

/// Map a function over a list
pub fn list_map<F>(list: &Value, f: F) -> Option<Value>
where
    F: Fn(&Value) -> Value,
{
    let elements = list_to_vec(list)?;
    let mapped: Vec<Value> = elements.iter().map(f).collect();
    Some(vec_to_list(mapped))
}

/// Filter a list
pub fn list_filter<F>(list: &Value, f: F) -> Option<Value>
where
    F: Fn(&Value) -> bool,
{
    let elements = list_to_vec(list)?;
    let filtered: Vec<Value> = elements.into_iter().filter(|v| f(v)).collect();
    Some(vec_to_list(filtered))
}

/// Fold a list from left to right
pub fn list_foldl<F, T>(list: &Value, init: T, f: F) -> Option<T>
where
    F: Fn(T, &Value) -> T,
{
    let elements = list_to_vec(list)?;
    let mut result = init;
    for element in elements.iter() {
        result = f(result, element);
    }
    Some(result)
}

/// Fold a list from right to left
pub fn list_foldr<F, T>(list: &Value, init: T, f: F) -> Option<T>
where
    F: Fn(&Value, T) -> T,
{
    let elements = list_to_vec(list)?;
    let mut result = init;
    for element in elements.iter().rev() {
        result = f(element, result);
    }
    Some(result)
}
