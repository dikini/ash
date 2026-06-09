use std::path::PathBuf;

use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};

use crate::ReplError;
use crate::completer::AshCompleter;

/// Helper struct for managing the readline editor.
#[derive(Debug)]
pub struct ReplEditor {
    editor: Editor<AshCompleter, rustyline::history::DefaultHistory>,
    history_path: Option<PathBuf>,
}

impl ReplEditor {
    pub fn new(history_path: Option<PathBuf>) -> Result<Self, ReplError> {
        let config = Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();

        let mut editor = Editor::with_config(config)?;
        editor.set_helper(Some(AshCompleter::new()));

        // Load history if path exists
        if let Some(path) = &history_path {
            #[allow(clippy::collapsible_if)]
            if path.exists() {
                editor.load_history(path).ok();
            }
        }

        Ok(Self {
            editor,
            history_path,
        })
    }

    pub fn readline(&mut self, prompt: &str) -> Result<String, ReadlineError> {
        self.editor.readline(prompt)
    }

    pub fn add_history_entry(&mut self, line: &str) {
        self.editor.add_history_entry(line).ok();
    }

    pub fn save_history(&mut self) {
        if let Some(path) = &self.history_path {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            self.editor.save_history(path).ok();
        }
    }
}
