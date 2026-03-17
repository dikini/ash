//! Token definitions for the Ash parser.
//!
//! This module defines all token types used by the Ash lexer and parser,
//! including keywords, literals, operators, delimiters, and identifiers.

/// Represents the kind of a token in the Ash language.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    /// `workflow` - Defines a workflow
    Workflow,
    /// `capability` - Defines a capability
    Capability,
    /// `policy` - Defines a policy
    Policy,
    /// `role` - Defines a role
    Role,

    // OODA loop keywords
    /// `observe` - Observation phase
    Observe,
    /// `orient` - Orientation phase
    Orient,
    /// `propose` - Proposal phase
    Propose,
    /// `decide` - Decision phase
    Decide,
    /// `act` - Action phase
    Act,

    // Control flow
    /// `oblige` - Obligation expression
    Oblige,
    /// `check` - Check expression
    Check,
    /// `let` - Variable binding
    Let,
    /// `if` - Conditional
    If,
    /// `then` - Then branch
    Then,
    /// `else` - Else branch
    Else,
    /// `for` - Loop construct
    For,
    /// `do` - Do block
    Do,
    /// `par` - Parallel block
    Par,
    /// `with` - With clause
    With,

    // Effect keywords
    /// `maybe` - Optional effect
    Maybe,
    /// `must` - Required effect
    Must,
    /// `attempt` - Attempt wrapper
    Attempt,
    /// `retry` - Retry wrapper
    Retry,
    /// `timeout` - Timeout wrapper
    Timeout,
    /// `done` - Completion marker
    Done,

    // Effect levels
    /// `epistemic` - Read-only effect level
    Epistemic,
    /// `deliberative` - Computation effect level
    Deliberative,
    /// `evaluative` - Comparison effect level
    Evaluative,
    /// `operational` - Side-effect level
    Operational,

    // Capability keywords
    /// `authority` - Authority declaration
    Authority,
    /// `obligations` - Obligations declaration
    Obligations,
    /// `supervises` - Supervision relation
    Supervises,

    // Type keywords
    /// `when` - When clause
    When,
    /// `returns` - Return type annotation
    Returns,
    /// `where` - Where clause
    Where,

    // Policy keywords
    /// `permit` - Permit decision
    Permit,
    /// `deny` - Deny decision
    Deny,
    /// `require_approval` - Approval requirement
    RequireApproval,
    /// `escalate` - Escalation action
    Escalate,

    // Operator keywords
    /// `in` - Membership test
    In,
    /// `not` - Logical negation (keyword form)
    Not,
    /// `and` - Logical conjunction (keyword form)
    And,
    /// `or` - Logical disjunction (keyword form)
    Or,

    // Boolean literals
    /// `true` - Boolean true
    True,
    /// `false` - Boolean false
    False,
    /// `null` - Null value
    Null,

    // Literals
    /// Integer literal (e.g., `42`, `-7`)
    Int(i64),
    /// Floating-point literal (e.g., `3.14`, `-0.5`)
    Float(f64),
    /// String literal (e.g., `"hello"`)
    String(Box<str>),
    /// Boolean literal value
    Bool(bool),

    // Operators
    /// `+` - Addition
    Plus,
    /// `-` - Subtraction or negation
    Minus,
    /// `*` - Multiplication
    Star,
    /// `/` - Division
    Slash,
    /// `=` - Assignment or equality (context-dependent)
    Eq,
    /// `!=` - Not equal
    Ne,
    /// `<` - Less than
    Lt,
    /// `>` - Greater than
    Gt,
    /// `<=` - Less than or equal
    Le,
    /// `>=` - Greater than or equal
    Ge,

    // Delimiters
    /// `(` - Left parenthesis
    LParen,
    /// `)` - Right parenthesis
    RParen,
    /// `{` - Left brace
    LBrace,
    /// `}` - Right brace
    RBrace,
    /// `[` - Left bracket
    LBracket,
    /// `]` - Right bracket
    RBracket,
    /// `,` - Comma
    Comma,
    /// `;` - Semicolon
    Semicolon,
    /// `:` - Colon
    Colon,
    /// `.` - Dot
    Dot,
    /// `..` - Range
    DotDot,

    // Other
    /// Identifier (e.g., `myVariable`, `some_function`)
    Ident(Box<str>),
    /// End of file
    Eof,
}

/// Represents a source code span with location information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Span {
    /// Byte offset from the start of the file
    pub start: usize,
    /// Byte offset of the end of the token
    pub end: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl Span {
    /// Creates a new span with the given parameters.
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

/// A token with its kind and source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,
    /// The source location of the token
    pub span: Span,
}

impl Token {
    /// Creates a new token with the given kind and span.
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Errors that can occur during lexical analysis.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LexError {
    /// Unexpected character encountered.
    #[error("unexpected character '{0}' at line {1}, column {2}")]
    UnexpectedChar(char, usize, usize),
    /// Unterminated string literal.
    #[error("unterminated string literal at line {0}, column {1}")]
    UnterminatedString(usize, usize),
    /// Invalid number format.
    #[error("invalid number format at line {0}, column {1}")]
    InvalidNumber(usize, usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_variants_exist() {
        // Test that all keyword variants can be constructed
        let _workflow = TokenKind::Workflow;
        let _capability = TokenKind::Capability;
        let _policy = TokenKind::Policy;
        let _role = TokenKind::Role;
        let _observe = TokenKind::Observe;
        let _orient = TokenKind::Orient;
        let _propose = TokenKind::Propose;
        let _decide = TokenKind::Decide;
        let _act = TokenKind::Act;
        let _oblige = TokenKind::Oblige;
        let _check = TokenKind::Check;
        let _let = TokenKind::Let;
        let _if = TokenKind::If;
        let _then = TokenKind::Then;
        let _else = TokenKind::Else;
        let _for = TokenKind::For;
        let _do = TokenKind::Do;
        let _par = TokenKind::Par;
        let _with = TokenKind::With;
        let _maybe = TokenKind::Maybe;
        let _must = TokenKind::Must;
        let _attempt = TokenKind::Attempt;
        let _retry = TokenKind::Retry;
        let _timeout = TokenKind::Timeout;
        let _done = TokenKind::Done;
        let _epistemic = TokenKind::Epistemic;
        let _deliberative = TokenKind::Deliberative;
        let _evaluative = TokenKind::Evaluative;
        let _operational = TokenKind::Operational;
        let _authority = TokenKind::Authority;
        let _obligations = TokenKind::Obligations;
        let _supervises = TokenKind::Supervises;
        let _when = TokenKind::When;
        let _returns = TokenKind::Returns;
        let _where = TokenKind::Where;
        let _permit = TokenKind::Permit;
        let _deny = TokenKind::Deny;
        let _require_approval = TokenKind::RequireApproval;
        let _escalate = TokenKind::Escalate;
        let _in = TokenKind::In;
        let _not = TokenKind::Not;
        let _and = TokenKind::And;
        let _or = TokenKind::Or;
        let _true = TokenKind::True;
        let _false = TokenKind::False;
        let _null = TokenKind::Null;

        // Test that keywords are distinct
        assert_ne!(_workflow, _capability);
        assert_ne!(_observe, _orient);
        assert_ne!(_epistemic, _operational);
        assert_ne!(_permit, _deny);
    }

    #[test]
    fn test_literal_variants() {
        let int_lit = TokenKind::Int(42);
        let float_lit = TokenKind::Float(1.5);
        let string_lit = TokenKind::String("hello".into());
        let bool_lit = TokenKind::Bool(true);
        let null_lit = TokenKind::Null;

        assert_eq!(int_lit, TokenKind::Int(42));
        assert_eq!(float_lit, TokenKind::Float(1.5));
        assert_eq!(string_lit, TokenKind::String("hello".into()));
        assert_eq!(bool_lit, TokenKind::Bool(true));
        assert_eq!(null_lit, TokenKind::Null);
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(0, 5, 1, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);

        let default_span = Span::default();
        assert_eq!(default_span.start, 0);
        assert_eq!(default_span.end, 0);
        assert_eq!(default_span.line, 0);
        assert_eq!(default_span.column, 0);
    }

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 20, 3, 5);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.line, 3);
        assert_eq!(span.column, 5);
    }

    #[test]
    fn test_token_creation() {
        let span = Span::new(0, 4, 1, 1);
        let token = Token::new(TokenKind::Workflow, span);
        assert_eq!(token.kind, TokenKind::Workflow);
        assert_eq!(token.span, span);

        let token2 = Token {
            kind: TokenKind::Ident("my_var".into()),
            span: Span::new(5, 11, 1, 6),
        };
        assert_eq!(token2.kind, TokenKind::Ident("my_var".into()));
        assert_eq!(token2.span.start, 5);
        assert_eq!(token2.span.end, 11);
    }

    #[test]
    fn test_lex_error_display_unexpected_char() {
        let err = LexError::UnexpectedChar('@', 10, 5);
        let display = format!("{}", err);
        assert_eq!(display, "unexpected character '@' at line 10, column 5");
    }

    #[test]
    fn test_lex_error_display_unterminated_string() {
        let err = LexError::UnterminatedString(5, 12);
        let display = format!("{}", err);
        assert_eq!(display, "unterminated string literal at line 5, column 12");
    }

    #[test]
    fn test_lex_error_display_invalid_number() {
        let err = LexError::InvalidNumber(8, 3);
        let display = format!("{}", err);
        assert_eq!(display, "invalid number format at line 8, column 3");
    }

    #[test]
    fn test_lex_error_equality() {
        let err1 = LexError::UnexpectedChar('x', 1, 2);
        let err2 = LexError::UnexpectedChar('x', 1, 2);
        let err3 = LexError::UnexpectedChar('y', 1, 2);
        let err4 = LexError::UnterminatedString(1, 2);

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
        assert_ne!(err1, err4);
    }

    #[test]
    fn test_operator_variants() {
        let ops = vec![
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Eq,
            TokenKind::Ne,
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::Le,
            TokenKind::Ge,
        ];

        // Ensure all operators are distinct
        for (i, op1) in ops.iter().enumerate() {
            for (j, op2) in ops.iter().enumerate() {
                if i != j {
                    assert_ne!(op1, op2);
                }
            }
        }
    }

    #[test]
    fn test_delimiter_variants() {
        let delims = vec![
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::Comma,
            TokenKind::Semicolon,
            TokenKind::Colon,
            TokenKind::Dot,
            TokenKind::DotDot,
        ];

        // Ensure all delimiters are distinct
        for (i, d1) in delims.iter().enumerate() {
            for (j, d2) in delims.iter().enumerate() {
                if i != j {
                    assert_ne!(d1, d2);
                }
            }
        }
    }

    #[test]
    fn test_ident_creation() {
        let ident = TokenKind::Ident("my_identifier".into());
        assert_eq!(ident, TokenKind::Ident("my_identifier".into()));

        let ident2 = TokenKind::Ident("_underscore".into());
        assert_eq!(ident2, TokenKind::Ident("_underscore".into()));
    }

    #[test]
    fn test_eof_variant() {
        let eof = TokenKind::Eof;
        assert_eq!(eof, TokenKind::Eof);
        assert_ne!(eof, TokenKind::Null);
    }

    #[test]
    fn test_token_clone() {
        let token = Token {
            kind: TokenKind::String("test".into()),
            span: Span::new(0, 6, 1, 1),
        };
        let cloned = token.clone();
        assert_eq!(token.kind, cloned.kind);
        assert_eq!(token.span, cloned.span);
    }

    #[test]
    fn test_span_copy() {
        let span = Span::new(5, 10, 2, 3);
        let copied = span;
        // Span implements Copy, so both should be usable
        assert_eq!(span.start, 5);
        assert_eq!(copied.start, 5);
    }
}
