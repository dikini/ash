//! Type definition parser for the Ash language.
//!
//! This module provides parsers for type definitions:
//! - `type Status = Pending | Processing | Completed;`
//! - `type Point = { x: Int, y: Int };`
//! - `type Option<T> = Some { value: T } | None;`
//! - `pub type Result<T, E> = Ok { value: T } | Err { error: E };`

use winnow::combinator::{alt, separated};
use winnow::prelude::*;
use winnow::stream::Stream;
use winnow::token::take_while;

use crate::input::ParseInput;
use crate::parse_utils::{keyword, literal_str, skip_whitespace_and_comments};

/// Type definition parsed from source
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    /// Name of the type being defined
    pub name: String,
    /// Type parameters for generic types
    pub params: Vec<String>,
    /// Body of the type definition
    pub body: TypeBody,
    /// Visibility of the type
    pub visibility: Visibility,
}

/// Body of a type definition
#[derive(Debug, Clone, PartialEq)]
pub enum TypeBody {
    /// Struct type: `type Point = { x: Int, y: Int };`
    Struct(Vec<(String, TypeExpr)>),
    /// Enum type: `type Status = Pending | Processing;`
    Enum(Vec<VariantDef>),
    /// Type alias: `type Name = String;`
    Alias(TypeExpr),
}

/// Variant definition for enums
#[derive(Debug, Clone, PartialEq)]
pub struct VariantDef {
    /// Name of the variant
    pub name: String,
    /// Fields of the variant (name, type pairs)
    pub fields: Vec<(String, TypeExpr)>,
}

/// Visibility modifier
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    /// Public visibility (accessible from anywhere)
    Public,
    /// Crate visibility (accessible within the crate)
    Crate,
    /// Private visibility (accessible only within the module)
    Private,
}

/// Type expression
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// Named type (e.g., Int, String, MyType)
    Named(String),
    /// Type constructor application (e.g., Option<Int>)
    Constructor { name: String, args: Vec<TypeExpr> },
    /// Tuple type (e.g., (Int, String))
    Tuple(Vec<TypeExpr>),
    /// Record type (e.g., { x: Int, y: String })
    Record(Vec<(String, TypeExpr)>),
}

/// Parse a type definition.
///
/// # Syntax
///
/// ```text
/// [pub] type Name[<T, U>] = TypeBody;
/// ```
///
/// # Examples
///
/// ```
/// use ash_parser::parse_type_def::parse_type_def;
/// use ash_parser::input::new_input;
/// use winnow::prelude::*;
///
/// let mut input = new_input("type Status = Pending | Processing;");
/// let result = parse_type_def(&mut input);
/// assert!(result.is_ok());
/// ```
pub fn parse_type_def(input: &mut ParseInput) -> ModalResult<TypeDef> {
    skip_whitespace_and_comments(input);

    // Parse optional visibility
    let visibility = parse_visibility(input)?;
    skip_whitespace_and_comments(input);

    // Parse "type" keyword
    keyword(input, "type")?;
    skip_whitespace_and_comments(input);

    // Parse type name
    let name = parse_type_name(input)?;
    skip_whitespace_and_comments(input);

    // Parse optional type parameters
    let params = parse_type_params(input)?;
    skip_whitespace_and_comments(input);

    // Parse "="
    literal_str("=").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse type body
    let body = parse_type_body(input)?;
    skip_whitespace_and_comments(input);

    // Parse trailing semicolon
    literal_str(";").parse_next(input)?;

    Ok(TypeDef {
        name: name.to_string(),
        params,
        body,
        visibility,
    })
}

/// Parse visibility modifier (pub, pub(crate), or private).
///
/// Returns `Visibility::Private` if no visibility modifier is present.
fn parse_visibility(input: &mut ParseInput) -> ModalResult<Visibility> {
    skip_whitespace_and_comments(input);

    // Try to match "pub" keyword
    if keyword(input, "pub").is_ok() {
        skip_whitespace_and_comments(input);
        // Check for pub(crate) syntax
        if literal_str("(").parse_next(input).is_ok() {
            skip_whitespace_and_comments(input);
            if keyword(input, "crate").is_ok() {
                skip_whitespace_and_comments(input);
                let _ = literal_str(")").parse_next(input);
                return Ok(Visibility::Crate);
            }
            // If not "crate", backtrack and treat as public
            return Ok(Visibility::Public);
        }
        Ok(Visibility::Public)
    } else {
        Ok(Visibility::Private)
    }
}

/// Parse a type name (identifier starting with uppercase).
fn parse_type_name<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    let name: &str =
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)?;

    // Check that first character is uppercase
    if !name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    Ok(name)
}

/// Parse type parameters: `<T, U>`
fn parse_type_params(input: &mut ParseInput) -> ModalResult<Vec<String>> {
    skip_whitespace_and_comments(input);

    // Check for opening `<`
    if literal_str("<").parse_next(input).is_err() {
        return Ok(vec![]);
    }

    let mut params = vec![];

    // Parse at least one type parameter
    skip_whitespace_and_comments(input);
    let first = parse_type_var(input)?;
    params.push(first.to_string());

    // Parse additional parameters
    loop {
        skip_whitespace_and_comments(input);
        if literal_str(",").parse_next(input).is_err() {
            break;
        }
        skip_whitespace_and_comments(input);
        let param = parse_type_var(input)?;
        params.push(param.to_string());
    }

    // Parse closing `>`
    skip_whitespace_and_comments(input);
    literal_str(">").parse_next(input)?;

    Ok(params)
}

/// Parse a type variable (identifier starting with uppercase).
fn parse_type_var<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    let name: &str =
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)?;

    // Check that first character is uppercase
    if !name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    Ok(name)
}

/// Parse type body (enum, struct, or alias).
fn parse_type_body(input: &mut ParseInput) -> ModalResult<TypeBody> {
    skip_whitespace_and_comments(input);

    // Try struct first (starts with {)
    if input.input.starts_with('{') {
        return parse_struct_body(input);
    }

    // Try enum - must have at least one | separator
    // We check by looking ahead for a | after the first variant
    if is_enum_body(input) {
        return parse_enum_body(input);
    }

    // Otherwise it's an alias
    parse_alias_body(input)
}

/// Check if this is an enum body by looking for a | separator
fn is_enum_body(input: &mut ParseInput) -> bool {
    // Save state
    let state = input.state;
    let checkpoint = input.input.checkpoint();

    // Try to parse first variant
    skip_whitespace_and_comments(input);
    if parse_type_name(input).is_err() {
        // Restore and return false
        input.state = state;
        input.input.reset(&checkpoint);
        return false;
    }

    skip_whitespace_and_comments(input);

    // Check for { after variant name (field block)
    if input.input.starts_with('{') {
        // Skip the field block
        let _ = literal_str("{").parse_next(input);
        skip_whitespace_and_comments(input);
        // Look for matching }
        let mut depth = 1;
        while depth > 0 && !input.input.is_empty() {
            if input.input.starts_with('{') {
                let _ = literal_str("{").parse_next(input);
                depth += 1;
            } else if input.input.starts_with('}') {
                let _ = literal_str("}").parse_next(input);
                depth -= 1;
            } else {
                let _ = input.input.next_token();
            }
        }
        skip_whitespace_and_comments(input);
    }

    // Check if there's a | after the first variant
    let is_enum = input.input.starts_with('|');

    // Restore state
    input.state = state;
    input.input.reset(&checkpoint);

    is_enum
}

/// Parse enum body: `Variant1 | Variant2 { field: Type }`
fn parse_enum_body(input: &mut ParseInput) -> ModalResult<TypeBody> {
    let variants = separated(1.., parse_variant, parse_variant_separator).parse_next(input)?;
    Ok(TypeBody::Enum(variants))
}

/// Parse variant separator `|`
fn parse_variant_separator(input: &mut ParseInput) -> ModalResult<()> {
    skip_whitespace_and_comments(input);
    literal_str("|").parse_next(input)?;
    skip_whitespace_and_comments(input);
    Ok(())
}

/// Parse a single enum variant.
fn parse_variant(input: &mut ParseInput) -> ModalResult<VariantDef> {
    skip_whitespace_and_comments(input);

    // Parse variant name
    let name = parse_type_name(input)?;
    skip_whitespace_and_comments(input);

    // Parse optional fields
    let fields = if literal_str("{").parse_next(input).is_ok() {
        let fields = parse_field_list(input)?;
        literal_str("}").parse_next(input)?;
        fields
    } else {
        vec![]
    };

    Ok(VariantDef {
        name: name.to_string(),
        fields,
    })
}

/// Parse struct body: `{ field: Type, ... }`
fn parse_struct_body(input: &mut ParseInput) -> ModalResult<TypeBody> {
    skip_whitespace_and_comments(input);
    literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let fields = parse_field_list(input)?;

    literal_str("}").parse_next(input)?;
    Ok(TypeBody::Struct(fields))
}

/// Parse a list of fields separated by commas.
fn parse_field_list(input: &mut ParseInput) -> ModalResult<Vec<(String, TypeExpr)>> {
    skip_whitespace_and_comments(input);

    // Empty record
    if literal_str("}").parse_next(input).is_ok() {
        return Ok(vec![]);
    }
    // Put the } back if we peeked it
    // Actually, we need to not consume it. Let's handle differently

    let mut fields = vec![];

    // Parse first field
    let (name, ty) = parse_field(input)?;
    fields.push((name.to_string(), ty));

    // Parse additional fields
    loop {
        skip_whitespace_and_comments(input);
        if literal_str(",").parse_next(input).is_err() {
            break;
        }
        skip_whitespace_and_comments(input);

        // Check for trailing comma by looking ahead
        skip_whitespace_and_comments(input);
        if input.input.starts_with('}') {
            break;
        }

        let (name, ty) = parse_field(input)?;
        fields.push((name.to_string(), ty));
    }

    Ok(fields)
}

/// Parse a single field: `name: Type`
fn parse_field(input: &mut ParseInput) -> ModalResult<(String, TypeExpr)> {
    skip_whitespace_and_comments(input);

    // Parse field name (lowercase)
    let name = parse_field_name(input)?;
    skip_whitespace_and_comments(input);

    // Parse ":"
    literal_str(":").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Parse type expression
    let ty = parse_type_expr(input)?;

    Ok((name.to_string(), ty))
}

/// Parse a field name (identifier starting with lowercase).
fn parse_field_name<'a>(input: &mut ParseInput<'a>) -> ModalResult<&'a str> {
    let name: &str =
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)?;

    // Check that first character is lowercase
    if name.is_empty() || !name.chars().next().unwrap().is_ascii_lowercase() {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    Ok(name)
}

/// Parse alias body: just a type expression
fn parse_alias_body(input: &mut ParseInput) -> ModalResult<TypeBody> {
    let ty = parse_type_expr(input)?;
    Ok(TypeBody::Alias(ty))
}

/// Parse a type expression.
fn parse_type_expr(input: &mut ParseInput) -> ModalResult<TypeExpr> {
    skip_whitespace_and_comments(input);

    alt((
        parse_tuple_type,
        parse_record_type,
        parse_constructor_type,
        parse_named_type,
    ))
    .parse_next(input)
}

/// Parse a named type: `Int`, `String`, `T`
fn parse_named_type(input: &mut ParseInput) -> ModalResult<TypeExpr> {
    let name = parse_type_name(input)?;
    Ok(TypeExpr::Named(name.to_string()))
}

/// Parse a type constructor: `Option<Int>`, `Result<T, E>`
fn parse_constructor_type(input: &mut ParseInput) -> ModalResult<TypeExpr> {
    let name = parse_type_name(input)?;
    skip_whitespace_and_comments(input);

    // Parse type arguments
    literal_str("<").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let args = separated(1.., parse_type_expr, parse_type_arg_separator).parse_next(input)?;

    skip_whitespace_and_comments(input);
    literal_str(">").parse_next(input)?;

    Ok(TypeExpr::Constructor {
        name: name.to_string(),
        args,
    })
}

/// Parse type argument separator
fn parse_type_arg_separator(input: &mut ParseInput) -> ModalResult<()> {
    skip_whitespace_and_comments(input);
    literal_str(",").parse_next(input)?;
    skip_whitespace_and_comments(input);
    Ok(())
}

/// Parse a tuple type: `(Int, String)`
fn parse_tuple_type(input: &mut ParseInput) -> ModalResult<TypeExpr> {
    literal_str("(").parse_next(input)?;
    skip_whitespace_and_comments(input);

    let elements = separated(1.., parse_type_expr, parse_type_arg_separator).parse_next(input)?;

    skip_whitespace_and_comments(input);
    literal_str(")").parse_next(input)?;

    Ok(TypeExpr::Tuple(elements))
}

/// Parse a record type: `{ x: Int, y: String }`
fn parse_record_type(input: &mut ParseInput) -> ModalResult<TypeExpr> {
    skip_whitespace_and_comments(input);
    literal_str("{").parse_next(input)?;
    skip_whitespace_and_comments(input);

    // Empty record
    if literal_str("}").parse_next(input).is_ok() {
        return Ok(TypeExpr::Record(vec![]));
    }
    // Put the } back if we peeked it - but we already consumed it

    let mut fields = vec![];

    // Parse first field
    let (name, ty) = parse_field(input)?;
    fields.push((name.to_string(), ty));

    // Parse additional fields
    loop {
        skip_whitespace_and_comments(input);
        if literal_str(",").parse_next(input).is_err() {
            break;
        }
        skip_whitespace_and_comments(input);

        // Check for trailing comma by looking ahead
        skip_whitespace_and_comments(input);
        if input.input.starts_with('}') {
            break;
        }

        let (name, ty) = parse_field(input)?;
        fields.push((name.to_string(), ty));
    }

    literal_str("}").parse_next(input)?;
    Ok(TypeExpr::Record(fields))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::new_input;

    #[test]
    fn test_parse_simple_enum() {
        let mut input = new_input("type Status = Pending | Processing;");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let type_def = result.unwrap();
        assert_eq!(type_def.name, "Status");
        assert!(type_def.params.is_empty());
        assert_eq!(type_def.visibility, Visibility::Private);

        match type_def.body {
            TypeBody::Enum(variants) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Pending");
                assert!(variants[0].fields.is_empty());
                assert_eq!(variants[1].name, "Processing");
                assert!(variants[1].fields.is_empty());
            }
            _ => panic!("Expected enum body"),
        }
    }

    #[test]
    fn test_parse_enum_with_fields() {
        let mut input = new_input("type Result = Ok { value: Int } | Err { error: String };");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let type_def = result.unwrap();
        assert_eq!(type_def.name, "Result");

        match type_def.body {
            TypeBody::Enum(variants) => {
                assert_eq!(variants.len(), 2);

                // Ok variant
                assert_eq!(variants[0].name, "Ok");
                assert_eq!(variants[0].fields.len(), 1);
                assert_eq!(variants[0].fields[0].0, "value");
                match &variants[0].fields[0].1 {
                    TypeExpr::Named(name) => assert_eq!(name, "Int"),
                    _ => panic!("Expected Named type"),
                }

                // Err variant
                assert_eq!(variants[1].name, "Err");
                assert_eq!(variants[1].fields.len(), 1);
                assert_eq!(variants[1].fields[0].0, "error");
                match &variants[1].fields[0].1 {
                    TypeExpr::Named(name) => assert_eq!(name, "String"),
                    _ => panic!("Expected Named type"),
                }
            }
            _ => panic!("Expected enum body"),
        }
    }

    #[test]
    fn test_parse_struct() {
        let mut input = new_input("type Point = { x: Int, y: Int };");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let type_def = result.unwrap();
        assert_eq!(type_def.name, "Point");

        match type_def.body {
            TypeBody::Struct(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert_eq!(fields[1].0, "y");
            }
            _ => panic!("Expected struct body"),
        }
    }

    #[test]
    fn test_parse_generic() {
        let mut input = new_input("type Option<T> = Some { value: T } | None;");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let type_def = result.unwrap();
        assert_eq!(type_def.name, "Option");
        assert_eq!(type_def.params, vec!["T"]);

        match type_def.body {
            TypeBody::Enum(variants) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Some");
                assert_eq!(variants[1].name, "None");
            }
            _ => panic!("Expected enum body"),
        }
    }

    #[test]
    fn test_parse_pub_visibility() {
        let mut input = new_input("pub type Inner = Int;");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let type_def = result.unwrap();
        eprintln!("Type def: {:?}", type_def);
        assert_eq!(type_def.name, "Inner");
        assert_eq!(type_def.visibility, Visibility::Public);

        match type_def.body {
            TypeBody::Alias(TypeExpr::Named(name)) => {
                assert_eq!(name, "Int");
            }
            _ => panic!("Expected alias body, got: {:?}", type_def.body),
        }
    }

    #[test]
    fn test_parse_generic_with_multiple_params() {
        let mut input = new_input("type Result<T, E> = Ok { value: T } | Err { error: E };");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let type_def = result.unwrap();
        assert_eq!(type_def.name, "Result");
        assert_eq!(type_def.params, vec!["T", "E"]);
    }

    #[test]
    fn test_parse_enum_with_unit_variants() {
        let mut input = new_input("type Status = Pending | Processing | Completed;");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let type_def = result.unwrap();
        match type_def.body {
            TypeBody::Enum(variants) => {
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].name, "Pending");
                assert_eq!(variants[1].name, "Processing");
                assert_eq!(variants[2].name, "Completed");
            }
            _ => panic!("Expected enum body"),
        }
    }

    #[test]
    fn test_parse_struct_with_trailing_comma() {
        let mut input = new_input("type Point = { x: Int, y: Int, };");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);

        let type_def = result.unwrap();
        match type_def.body {
            TypeBody::Struct(fields) => {
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected struct body"),
        }
    }

    #[test]
    fn test_parse_type_with_whitespace() {
        let mut input = new_input("type   Status   =   Pending   |   Processing   ;");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);
    }

    #[test]
    fn test_parse_type_with_newlines() {
        let mut input = new_input("type Status =\n  Pending\n  | Processing;");
        let result = parse_type_def(&mut input);
        assert!(result.is_ok(), "Parse failed: {:?}", result);
    }
}
