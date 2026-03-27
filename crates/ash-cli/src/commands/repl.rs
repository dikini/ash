//! REPL command for interactive Ash workflow evaluation.

use anyhow::Result;
use ash_repl::ReplConfig;
use clap::Args;
use std::path::PathBuf;

/// Arguments for the REPL command.
#[derive(Args, Debug, Clone)]
pub struct ReplArgs {
    /// Override the history file path.
    #[arg(long, value_name = "PATH")]
    pub history: Option<PathBuf>,

    /// Disable history load/save for this session.
    #[arg(long)]
    pub no_history: bool,

    /// Initialize with file
    #[arg(long, value_name = "FILE")]
    pub init: Option<PathBuf>,

    /// Config file path
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

impl ReplArgs {
    fn to_config(&self) -> ReplConfig {
        if self.no_history {
            ReplConfig::no_history()
        } else if let Some(path) = &self.history {
            ReplConfig::with_history_path(path.clone())
        } else {
            ReplConfig::with_default_history()
        }
    }
}

/// Run the interactive REPL.
pub async fn repl(args: &ReplArgs) -> Result<()> {
    ash_repl::run_with_config(args.to_config())
        .await
        .map_err(anyhow::Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_args_parsing() {
        let args = ReplArgs {
            history: Some(PathBuf::from(".my_history")),
            no_history: false,
            init: Some(PathBuf::from("init.ash")),
            config: Some(PathBuf::from("config.toml")),
        };

        assert_eq!(args.history, Some(PathBuf::from(".my_history")));
        assert!(!args.no_history);
        assert_eq!(args.init, Some(PathBuf::from("init.ash")));
        assert_eq!(args.config, Some(PathBuf::from("config.toml")));
    }

    #[test]
    fn test_repl_args_defaults() {
        let args = ReplArgs {
            history: None,
            no_history: false,
            init: None,
            config: None,
        };

        assert_eq!(args.history, None);
        assert!(!args.no_history);
        assert_eq!(args.init, None);
        assert_eq!(args.config, None);
    }

    #[test]
    fn test_repl_args_convert_to_history_override() {
        let args = ReplArgs {
            history: Some(PathBuf::from("/tmp/ash-history")),
            no_history: false,
            init: None,
            config: None,
        };

        assert_eq!(
            args.to_config().history_path(),
            Some(&PathBuf::from("/tmp/ash-history"))
        );
    }

    #[test]
    fn test_no_history_takes_precedence_over_history_override() {
        let args = ReplArgs {
            history: Some(PathBuf::from("/tmp/ash-history")),
            no_history: true,
            init: None,
            config: None,
        };

        assert_eq!(args.to_config(), ReplConfig::no_history());
    }
}
