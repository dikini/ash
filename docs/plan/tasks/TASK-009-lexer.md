# TASK-009: Lexer Implementation

## Status: ✅ Complete

## Description

Implement the Ash lexer that transforms source text into a stream of tokens with span information and error recovery.

## Specification Reference

- SPEC-002: Surface Language - Section 2. Lexical Structure

## Requirements

### Functional Requirements

1. Tokenize all keywords: `workflow`, `capability`, `policy`, `role`, `observe`, `orient`, `propose`, `decide`, `act`, `oblige`, `check`, `let`, `if`, `then`, `else`, `for`, `do`, `par`, `with`, `maybe`, `must`, `attempt`, `retry`, `timeout`, `done`, `epistemic`, `deliberative`, `evaluative`, `operational`, `authority`, `obligations`, `supervises`, `when`, `returns`, `where`, `permit`, `deny`, `require_approval`, `escalate`

2. Tokenize literals:
   - Integers: `[0-9]+`
   - Floats: `[0-9]+\.[0-9]+`
   - Strings: `"[^"]*"`
   - Bools: `true`, `false`
   - Null: `null`

3. Tokenize operators: `+`, `-`, `*`, `/`, `=`, `!=`, `<`, `>`, `<=`, `>=`, `and`, `or`, `not`, `in`

4. Tokenize delimiters: `(`, `)`, `{`, `}`, `[`, `]`, `,`, `;`, `:`, `.`, `..`

5. Handle identifiers: `[a-zA-Z_][a-zA-Z0-9_-]*`

6. Handle comments:
   - Line comments: `--` to end of line
   - Block comments: `/* */` (nested not required for MVP)
   - Doc comments: `-- |` to end of line

7. Skip whitespace and track line/column information

### Error Handling

- `LexError` type with span information
- Error for unterminated string literals
- Error for invalid characters
- Error for malformed numbers

## TDD Steps

### Step 1: Write Tests for Basic Tokenization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_tokenization() {
        let input = "workflow observe act done";
        let tokens = lex(input).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Workflow);
        assert_eq!(tokens[1].kind, TokenKind::Observe);
        assert_eq!(tokens[2].kind, TokenKind::Act);
        assert_eq!(tokens[3].kind, TokenKind::Done);
    }

    #[test]
    fn test_identifier_tokenization() {
        let input = "foo_bar baz123";
        let tokens = lex(input).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident("foo_bar".into()));
        assert_eq!(tokens[1].kind, TokenKind::Ident("baz123".into()));
    }

    #[test]
    fn test_literal_tokenization() {
        let input = r#"42 3.14 "hello" true null"#;
        let tokens = lex(input).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Int(42));
        assert_eq!(tokens[1].kind, TokenKind::Float(3.14));
        assert_eq!(tokens[2].kind, TokenKind::String("hello".into()));
        assert_eq!(tokens[3].kind, TokenKind::Bool(true));
        assert_eq!(tokens[4].kind, TokenKind::Null);
    }
}
```

### Step 2: Implement Token Types

Create `crates/ash-parser/src/token.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Workflow, Capability, Policy, Role,
    Observe, Orient, Propose, Decide, Act, Oblige, Check,
    Let, If, Then, Else, For, Do, Par, With, Maybe, Must,
    Attempt, Retry, Timeout, Done,
    Epistemic, Deliberative, Evaluative, Operational,
    Authority, Obligations, Supervises,
    When, Returns, Where,
    Permit, Deny, RequireApproval, Escalate,
    
    // Literals
    Int(i64),
    Float(f64),
    String(Box<str>),
    Bool(bool),
    Null,
    
    // Operators
    Plus, Minus, Star, Slash,
    Eq, Neq, Lt, Gt, Leq, Geq,
    And, Or, Not, In,
    
    // Delimiters
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Semicolon, Colon, Dot, DotDot,
    
    // Other
    Ident(Box<str>),
    
    // Meta
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,  // byte offset
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
```

### Step 3: Implement Core Lexer

Create `crates/ash-parser/src/lexer.rs`:

```rust
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return Ok(self.make_token(TokenKind::Eof));
        }
        
        match self.peek() {
            '(' => { self.advance(); Ok(self.make_token(TokenKind::LParen)) }
            ')' => { self.advance(); Ok(self.make_token(TokenKind::RParen)) }
            // ... more single-char tokens
            '"' => self.read_string(),
            '-' if self.peek_next() == Some('-') => {
                self.skip_comment();
                self.next_token()
            }
            c if c.is_ascii_digit() => self.read_number(),
            c if c.is_ascii_alphabetic() || c == '_' => self.read_word(),
            c => Err(self.error(format!("unexpected character: {}", c))),
        }
    }
}
```

### Step 4: Implement Iterator Interface

```rust
impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(tok) if tok.kind == TokenKind::Eof => None,
            result => Some(result),
        }
    }
}
```

### Step 5: Add Error Recovery

```rust
pub fn lex_with_recovery(input: &str) -> (Vec<Token>, Vec<LexError>) {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    
    loop {
        match lexer.next_token() {
            Ok(tok) if tok.kind == TokenKind::Eof => break,
            Ok(tok) => tokens.push(tok),
            Err(e) => {
                errors.push(e);
                lexer.skip_to_next_token();
            }
        }
    }
    
    (tokens, errors)
}
```

### Step 6: Add Property Tests

```rust
proptest! {
    #[test]
    fn prop_roundtrip_identifier(s in "[a-zA-Z_][a-zA-Z0-9_-]{0,20}") {
        let tokens = lex(&s).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Ident(s.into()));
    }
    
    #[test]
    fn prop_int_literal(n in any::<i64>()) {
        let s = n.to_string();
        let tokens = lex(&s).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Int(n));
    }
}
```

## Completion Checklist

- [ ] All keywords tokenized correctly
- [ ] All literals parsed correctly (including escape sequences in strings)
- [ ] All operators recognized
- [ ] All delimiters recognized
- [ ] Span tracking (line, column, byte offset)
- [ ] Error recovery (continue lexing after error)
- [ ] Property tests for identifiers and literals
- [ ] Unit tests for edge cases
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes
- [ ] Documentation complete

## Self-Review Questions

1. **Performance**: Is the lexer zero-copy where possible?
   - String literals should reference source text (Cow or similar)

2. **Correctness**: Are all SPEC-002 lexical rules implemented?
   - Cross-reference with specification

3. **Error messages**: Are they actionable?
   - Include line/column, snippet, and suggestion

## Estimated Effort

6 hours

## Dependencies

- TASK-008: Token definitions (uses TokenKind, Span)

## Blocked By

- TASK-008: Token definitions

## Blocks

- TASK-010: Lexer property tests
- TASK-012: Parser core (needs working lexer)
