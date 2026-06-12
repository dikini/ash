//! Ash MCP server binary entry point.
//!
//! Launch via `ash lsp --mcp` (per SPEC-005) or directly as `ash-mcp`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Handle CLI flags before initializing tracing or the runtime so that
    // --version and --help produce clean output on stdout and exit 0.
    // This is intentionally minimal; if more flags are added later, migrate to
    // `clap` (see rust-skills api-builder-pattern).
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        match args[1].as_str() {
            "--version" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" => {
                println!("ash-mcp -- Ash MCP server");
                println!();
                println!("Usage: ash-mcp [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --version    Print version and exit");
                println!("  --help       Print this help message and exit");
                println!(
                    "  --daemon     Run in persistent daemon mode (line-delimited JSON-RPC on stdin)"
                );
                println!();
                println!("When run without flags, the server starts in stdio MCP mode.");
                return Ok(());
            }
            "--daemon" => {
                // Daemon mode: no async runtime needed, process line-delimited JSON-RPC
                tracing_subscriber::fmt()
                    .with_writer(std::io::stderr)
                    .with_target(false)
                    .init();

                let state = ash_mcp::daemon::DaemonState::new();
                let stdin = std::io::stdin().lock();
                let stdout = std::io::stdout().lock();
                return ash_mcp::daemon::run_daemon_loop(&state, stdin, stdout);
            }
            _ => {}
        }
    }

    // Initialize tracing to stderr so it doesn't interfere with stdio MCP transport.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { ash_mcp::run_stdio().await })
}

// ---------------------------------------------------------------------------
// Binary-level tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::process::Command;

    fn binary_path() -> std::path::PathBuf {
        // Cargo sets CARGO_BIN_EXE_ash-mcp when running integration-style tests.
        std::env::var_os("CARGO_BIN_EXE_ash-mcp").map_or_else(
            || {
                // Fallback: walk up from the test executable to the target profile
                // directory, where the binary artifact lives.
                let mut path = std::env::current_exe().expect("current exe");
                path.pop(); // deps (or the exe itself on some platforms)
                if path.file_name() == Some(std::ffi::OsStr::new("deps")) {
                    path.pop(); // debug / release
                }
                path.push("ash-mcp");
                assert!(path.exists(), "ash-mcp binary not found at {path:?}");
                path
            },
            std::path::PathBuf::from,
        )
    }

    #[test]
    fn test_cli_version_flag() {
        let output = Command::new(binary_path())
            .arg("--version")
            .output()
            .expect("spawn ash-mcp --version");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "--version should exit 0\nstdout: {stdout}\nstderr: {stderr}"
        );
        let expected_version = env!("CARGO_PKG_VERSION");
        assert!(
            stdout.contains(expected_version),
            "--version should print workspace version {expected_version}, got: {stdout}"
        );
    }

    #[test]
    fn test_cli_help_flag() {
        let output = Command::new(binary_path())
            .arg("--help")
            .output()
            .expect("spawn ash-mcp --help");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "--help should exit 0\nstdout: {stdout}\nstderr: {stderr}"
        );
        assert!(
            stdout.to_lowercase().contains("usage") || stdout.to_lowercase().contains("ash-mcp"),
            "--help should print usage info, got: {stdout}"
        );
    }

    #[test]
    fn test_stdio_no_stray_stdout_on_launch() {
        // Launch the binary and immediately close stdin so it exits quickly.
        let mut child = Command::new(binary_path())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn ash-mcp");

        // Give it a moment to start up and then terminate.
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = child.kill();
        let output = child.wait_with_output().expect("wait on child");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.trim().is_empty(),
            "stdio transport must not emit stray stdout, got: {stdout}"
        );
    }
}
