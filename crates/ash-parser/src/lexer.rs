//! Lexer for the Ash parser.
//!
//! This module provides lexical analysis for the Ash workflow language,
//! transforming source text into a stream of tokens.

use crate::token::{LexError, Span, Token, TokenKind};

/// The lexer for the Ash language.
///
/// Transforms source text into tokens by iterating through the input
/// character by character and recognizing token patterns.
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::str::Chars<'a>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given input string.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Returns the next token from the input.
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // Skip whitespace and comments
        self.skip_whitespace_and_comments();

        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        // Get the next character
        let ch = match self.peek_char() {
            Some(c) => c,
            None => {
                // End of file
                return Ok(Token::new(
                    TokenKind::Eof,
                    Span::new(start_pos, start_pos, start_line, start_col),
                ));
            }
        };

        let kind = match ch {
            // Single-character operators and delimiters
            '+' => {
                self.advance();
                TokenKind::Plus
            }
            '-' => {
                self.advance();
                TokenKind::Minus
            }
            '*' => {
                self.advance();
                TokenKind::Star
            }
            '/' => {
                self.advance();
                TokenKind::Slash
            }
            '(' => {
                self.advance();
                TokenKind::LParen
            }
            ')' => {
                self.advance();
                TokenKind::RParen
            }
            '{' => {
                self.advance();
                TokenKind::LBrace
            }
            '}' => {
                self.advance();
                TokenKind::RBrace
            }
            '[' => {
                self.advance();
                TokenKind::LBracket
            }
            ']' => {
                self.advance();
                TokenKind::RBracket
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            ';' => {
                self.advance();
                TokenKind::Semicolon
            }
            ':' => {
                self.advance();
                TokenKind::Colon
            }

            // Two-character operators and delimiters
            '=' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    // Should be Eq for single =
                    // But let's check if there's another = for ==
                    // Actually, according to spec, = is Eq and != is Ne
                    // So just = is fine
                    TokenKind::Eq
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::Ne
                } else {
                    // '!' alone is unexpected
                    return Err(LexError::UnexpectedChar('!', start_line, start_col));
                }
            }
            '<' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '.' => {
                self.advance();
                if self.peek_char() == Some('.') {
                    self.advance();
                    TokenKind::DotDot
                } else {
                    TokenKind::Dot
                }
            }

            // String literal
            '"' => self.read_string(start_line, start_col)?,

            // Number literal (integer or float)
            '0'..='9' => self.read_number()?, // Line 175

            // Identifier or keyword
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier_or_keyword(),

            // Unknown character
            _ => {
                self.advance();
                return Err(LexError::UnexpectedChar(ch, start_line, start_col));
            }
        };

        let end_pos = self.position;
        let span = Span::new(start_pos, end_pos, start_line, start_col);
        Ok(Token::new(kind, span))
    }

    /// Peeks at the next character without consuming it.
    fn peek_char(&self) -> Option<char> {
        self.chars.clone().next()
    }

    /// Advances to the next character and returns it.
    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next();
        if let Some(c) = ch {
            self.position += c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    /// Skips whitespace and comments.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek_char() {
                Some(' ') | Some('\t') | Some('\n') | Some('\r') => {
                    self.advance();
                }
                Some('-') => {
                    // Check for line comment --
                    let mut chars_clone = self.chars.clone();
                    chars_clone.next();
                    if chars_clone.next() == Some('-') {
                        // Line comment, skip until newline or EOF
                        self.advance();
                        self.advance();
                        loop {
                            match self.peek_char() {
                                Some('\n') | None => break,
                                _ => {
                                    self.advance();
                                }
                            }
                        }
                    } else {
                        // Not a comment, this is likely a minus operator
                        break;
                    }
                }
                Some('/') => {
                    // Check for block comment /*
                    let mut chars_clone = self.chars.clone();
                    chars_clone.next();
                    if chars_clone.next() == Some('*') {
                        // Block comment, skip until */
                        self.advance();
                        self.advance();
                        let mut prev_char = ' ';
                        loop {
                            match self.peek_char() {
                                Some('*') => {
                                    prev_char = '*';
                                    self.advance();
                                }
                                Some('/') => {
                                    if prev_char == '*' {
                                        self.advance();
                                        break;
                                    } else {
                                        prev_char = '/';
                                        self.advance();
                                    }
                                }
                                Some(c) => {
                                    prev_char = c;
                                    self.advance();
                                }
                                None => break, // Unterminated block comment
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    /// Reads a string literal (content between double quotes).
    fn read_string(&mut self, start_line: usize, start_col: usize) -> Result<TokenKind, LexError> {
        // Consume the opening quote
        self.advance();

        let start_pos = self.position;
        let mut end_pos = start_pos;

        loop {
            match self.peek_char() {
                Some('"') => {
                    // Found closing quote
                    break;
                }
                Some('\n') | None => {
                    // Unterminated string
                    return Err(LexError::UnterminatedString(start_line, start_col));
                }
                Some(_) => {
                    end_pos = self.position + 1;
                    self.advance();
                }
            }
        }

        // Extract the string content
        let content = &self.input[start_pos..end_pos];

        // Consume the closing quote
        self.advance();

        Ok(TokenKind::String(content.into()))
    }

    /// Reads a number literal (integer or float).
    fn read_number(&mut self) -> Result<TokenKind, LexError> {
        let start_pos = self.position;

        // Read integer part
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        // Check for decimal point
        if self.peek_char() == Some('.') {
            // Lookahead to check if this is a float
            let mut chars_clone = self.chars.clone();
            chars_clone.next(); // Skip the '.'
            if let Some(next_ch) = chars_clone.next()
                && next_ch.is_ascii_digit()
            {
                // It's a float
                self.advance(); // Consume '.'

                // Read fractional part
                while let Some(ch) = self.peek_char() {
                    if ch.is_ascii_digit() {
                        self.advance();
                    } else {
                        break;
                    }
                }

                let num_str = &self.input[start_pos..self.position];
                match num_str.parse::<f64>() {
                    Ok(f) => return Ok(TokenKind::Float(f)),
                    Err(_) => return Err(LexError::InvalidNumber(self.line, self.column)),
                }
            }
        }

        // It's an integer
        let num_str = &self.input[start_pos..self.position];
        match num_str.parse::<i64>() {
            Ok(i) => Ok(TokenKind::Int(i)),
            Err(_) => Err(LexError::InvalidNumber(self.line, self.column)),
        }
    }

    /// Reads an identifier or keyword.
    fn read_identifier_or_keyword(&mut self) -> TokenKind {
        let start_pos = self.position;

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                self.advance();
            } else {
                break;
            }
        }

        let ident = &self.input[start_pos..self.position];
        lookup_keyword(ident)
    }
}

/// Looks up a string and returns the corresponding TokenKind.
/// Returns TokenKind::Ident if the string is not a keyword.
fn lookup_keyword(ident: &str) -> TokenKind {
    match ident {
        // Workflow keywords
        "workflow" => TokenKind::Workflow,
        "capability" => TokenKind::Capability,
        "policy" => TokenKind::Policy,
        "role" => TokenKind::Role,

        // OODA loop keywords
        "observe" => TokenKind::Observe,
        "orient" => TokenKind::Orient,
        "propose" => TokenKind::Propose,
        "decide" => TokenKind::Decide,
        "act" => TokenKind::Act,

        // Control flow
        "oblige" => TokenKind::Oblige,
        "check" => TokenKind::Check,
        "let" => TokenKind::Let,
        "if" => TokenKind::If,
        "then" => TokenKind::Then,
        "else" => TokenKind::Else,
        "for" => TokenKind::For,
        "do" => TokenKind::Do,
        "par" => TokenKind::Par,
        "with" => TokenKind::With,

        // Effect keywords
        "maybe" => TokenKind::Maybe,
        "must" => TokenKind::Must,
        "attempt" => TokenKind::Attempt,
        "retry" => TokenKind::Retry,
        "timeout" => TokenKind::Timeout,
        "done" => TokenKind::Done,
        "ret" => TokenKind::Ret,

        // Effect levels
        "epistemic" => TokenKind::Epistemic,
        "deliberative" => TokenKind::Deliberative,
        "evaluative" => TokenKind::Evaluative,
        "operational" => TokenKind::Operational,

        // Capability keywords
        "authority" => TokenKind::Authority,
        "obligations" => TokenKind::Obligations,

        // Type keywords
        "when" => TokenKind::When,
        "returns" => TokenKind::Returns,
        "where" => TokenKind::Where,

        // Policy keywords
        "permit" => TokenKind::Permit,
        "deny" => TokenKind::Deny,
        "require_approval" => TokenKind::RequireApproval,
        "escalate" => TokenKind::Escalate,

        // Operator keywords
        "in" => TokenKind::In,
        "not" => TokenKind::Not,
        "and" => TokenKind::And,
        "or" => TokenKind::Or,

        // Boolean literals
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "null" => TokenKind::Null,

        // Not a keyword, return as identifier
        _ => TokenKind::Ident(ident.into()),
    }
}

/// Convenience function to lex an entire input string.
///
/// Returns all tokens including the final Eof token, or the first error encountered.
pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token()?;
        let is_eof = token.kind == TokenKind::Eof;
        tokens.push(token);
        if is_eof {
            break;
        }
    }

    Ok(tokens)
}

/// Lex with error recovery - continues lexing even after errors.
///
/// Returns a tuple of (tokens, errors). All non-error tokens are collected,
/// and errors are recorded separately.
pub fn lex_with_recovery(input: &str) -> (Vec<Token>, Vec<LexError>) {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    loop {
        match lexer.next_token() {
            Ok(token) => {
                let is_eof = token.kind == TokenKind::Eof;
                tokens.push(token);
                if is_eof {
                    break;
                }
            }
            Err(err) => {
                errors.push(err);
            }
        }
    }

    (tokens, errors)
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_tokenization() {
        let tokens = lex("workflow observe act done").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Workflow);
        assert_eq!(tokens[1].kind, TokenKind::Observe);
        assert_eq!(tokens[2].kind, TokenKind::Act);
        assert_eq!(tokens[3].kind, TokenKind::Done);
    }

    #[test]
    fn test_identifier_tokenization() {
        let tokens = lex("myVariable my_function _private supervises").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident("myVariable".into()));
        assert_eq!(tokens[1].kind, TokenKind::Ident("my_function".into()));
        assert_eq!(tokens[2].kind, TokenKind::Ident("_private".into()));
        assert_eq!(tokens[3].kind, TokenKind::Ident("supervises".into()));
    }

    #[test]
    fn test_literal_tokenization() {
        let tokens = lex("42 3.14 \"hello\" true false null").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Int(42));
        assert_eq!(tokens[1].kind, TokenKind::Float(3.14));
        assert_eq!(tokens[2].kind, TokenKind::String("hello".into()));
        assert_eq!(tokens[3].kind, TokenKind::True);
        assert_eq!(tokens[4].kind, TokenKind::False);
        assert_eq!(tokens[5].kind, TokenKind::Null);
    }

    #[test]
    fn test_operator_tokenization() {
        let tokens = lex("+ - * / = != < > <= >= ..").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::Eq);
        assert_eq!(tokens[5].kind, TokenKind::Ne);
        assert_eq!(tokens[6].kind, TokenKind::Lt);
        assert_eq!(tokens[7].kind, TokenKind::Gt);
        assert_eq!(tokens[8].kind, TokenKind::Le);
        assert_eq!(tokens[9].kind, TokenKind::Ge);
        assert_eq!(tokens[10].kind, TokenKind::DotDot);
    }

    #[test]
    fn test_comment_skipping() {
        // Line comments
        let tokens = lex("workflow -- this is a comment\nobserve").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Workflow);
        assert_eq!(tokens[1].kind, TokenKind::Observe);

        // Block comments
        let tokens = lex("workflow /* block comment */ observe").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Workflow);
        assert_eq!(tokens[1].kind, TokenKind::Observe);

        // Nested/complex block comments
        let tokens = lex("/* outer */ workflow /* inner */ observe /* end */").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Workflow);
        assert_eq!(tokens[1].kind, TokenKind::Observe);
    }

    #[test]
    fn test_span_tracking() {
        let input = "workflow\n  observe";
        let tokens = lex(input).unwrap();

        // First token on line 1 ("workflow" is 8 bytes: positions 0-7, end=8)
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 8);

        // Second token on line 2 (after newline and 2 spaces: positions 8, 9, 10)
        // "observe" is 7 bytes: positions 11-17, end=18
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[1].span.column, 3);
        assert_eq!(tokens[1].span.start, 11);
        assert_eq!(tokens[1].span.end, 18);
    }

    #[test]
    fn test_error_recovery() {
        let (tokens, errors) = lex_with_recovery("workflow @ observe # act");

        // Should have workflow, observe, act tokens plus Eof
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Workflow);
        assert_eq!(tokens[1].kind, TokenKind::Observe);
        assert_eq!(tokens[2].kind, TokenKind::Act);

        // Should have 2 errors for @ and #
        assert_eq!(errors.len(), 2);
        assert!(matches!(errors[0], LexError::UnexpectedChar('@', 1, 10)));
        assert!(matches!(errors[1], LexError::UnexpectedChar('#', 1, 20)));
    }

    #[test]
    fn test_all_keywords() {
        let input = r#"
            workflow capability policy role
            observe orient propose decide act
            oblige check let if then else for do par with
            maybe must attempt retry timeout done
            epistemic deliberative evaluative operational
            authority obligations
            when returns where
            permit deny require_approval escalate
            in not and or
            true false null
        "#;

        let tokens = lex(input).unwrap();
        // Filter out Eof, check that all keywords are recognized
        let keyword_tokens: Vec<_> = tokens.iter().filter(|t| t.kind != TokenKind::Eof).collect();

        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::Workflow));
        assert!(
            keyword_tokens
                .iter()
                .any(|t| t.kind == TokenKind::Capability)
        );
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::Policy));
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::Role));
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::Observe));
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::Orient));
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::Propose));
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::Decide));
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::Act));
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::True));
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::False));
        assert!(keyword_tokens.iter().any(|t| t.kind == TokenKind::Null));
    }

    #[test]
    fn test_delimiters() {
        let tokens = lex("( ) { } [ ] , ; : .").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::LParen);
        assert_eq!(tokens[1].kind, TokenKind::RParen);
        assert_eq!(tokens[2].kind, TokenKind::LBrace);
        assert_eq!(tokens[3].kind, TokenKind::RBrace);
        assert_eq!(tokens[4].kind, TokenKind::LBracket);
        assert_eq!(tokens[5].kind, TokenKind::RBracket);
        assert_eq!(tokens[6].kind, TokenKind::Comma);
        assert_eq!(tokens[7].kind, TokenKind::Semicolon);
        assert_eq!(tokens[8].kind, TokenKind::Colon);
        assert_eq!(tokens[9].kind, TokenKind::Dot);
    }

    #[test]
    fn test_integer_literals() {
        let tokens = lex("0 1 42 1234567890").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Int(0));
        assert_eq!(tokens[1].kind, TokenKind::Int(1));
        assert_eq!(tokens[2].kind, TokenKind::Int(42));
        assert_eq!(tokens[3].kind, TokenKind::Int(1234567890));
    }

    #[test]
    fn test_float_literals() {
        let tokens = lex("0.0 1.5 3.14159 123.456").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Float(0.0));
        assert_eq!(tokens[1].kind, TokenKind::Float(1.5));
        assert_eq!(tokens[2].kind, TokenKind::Float(3.14159));
        assert_eq!(tokens[3].kind, TokenKind::Float(123.456));
    }

    #[test]
    fn test_string_literals() {
        let tokens = lex("\"hello\" \"world\" \"with spaces\"").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::String("hello".into()));
        assert_eq!(tokens[1].kind, TokenKind::String("world".into()));
        assert_eq!(tokens[2].kind, TokenKind::String("with spaces".into()));
    }

    #[test]
    fn test_unterminated_string_error() {
        let result = lex("\"unterminated");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LexError::UnterminatedString(_, _)
        ));
    }

    #[test]
    fn test_unexpected_char_error() {
        let result = lex("@");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LexError::UnexpectedChar('@', 1, 1)
        ));
    }

    #[test]
    fn test_identifier_with_hyphens() {
        let tokens = lex("my-identifier with-multiple-hyphens").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident("my-identifier".into()));
        assert_eq!(
            tokens[1].kind,
            TokenKind::Ident("with-multiple-hyphens".into())
        );
    }

    #[test]
    fn test_eof_token() {
        let tokens = lex("").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }
}
