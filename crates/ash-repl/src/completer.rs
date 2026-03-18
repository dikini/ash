//! Tab completion for the REPL.

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;

/// Completer for Ash language.
#[derive(Debug, Clone, Default)]
pub struct AshCompleter;

impl AshCompleter {
    /// Create a new completer.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Completer for AshCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Basic keyword completion - will be expanded in TASK-081
        let keywords = [
            "workflow",
            "action",
            "capability",
            "effect",
            "let",
            "if",
            "then",
            "else",
        ];

        let start = line[..pos]
            .rfind(|c: char| c.is_whitespace())
            .map_or(0, |i| i + 1);
        let prefix = &line[start..pos];

        let matches: Vec<Pair> = keywords
            .iter()
            .filter(|kw| kw.starts_with(prefix))
            .map(|kw| Pair {
                display: (*kw).to_string(),
                replacement: (*kw).to_string(),
            })
            .collect();

        Ok((start, matches))
    }
}

impl Highlighter for AshCompleter {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Cow::Owned(format!("\x1b[1;32m{prompt}\x1b[0m"))
        } else {
            Cow::Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[1;30m{hint}\x1b[0m"))
    }
}

impl Validator for AshCompleter {}

impl Hinter for AshCompleter {
    type Hint = String;
}

impl Helper for AshCompleter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completer_creates() {
        let completer = AshCompleter::new();
        // Just verify it can be created
        let _ = completer;
    }
}
