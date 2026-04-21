//! Capability boundary audit for the Ash spec processor.
//!
//! Declares the set of capabilities the Ash runtime is expected to provide
//! (regex, JSON, Markdown, file I/O, stdio, process) and audits the stdlib
//! tree to confirm the corresponding `.ash` stub files exist and contain
//! meaningful declarations. Capabilities that are not yet implemented produce
//! informational findings so the gap is tracked.

use std::fs;
use std::path::Path;

use crate::finding::SpecFinding;

/// A single capability that the Ash runtime declares.
#[derive(Debug, Clone)]
pub struct Capability {
    /// Machine-readable name, e.g. `"regex"`, `"json"`.
    pub name: String,
    /// Whether the capability is expected to be present.
    pub expected: bool,
    /// Path to the stdlib `.ash` file relative to the repo root,
    /// e.g. `"std/src/regex.ash"`.
    pub stdlib_file: String,
}

/// Returns the full boundary declaration — the authoritative list of
/// capabilities the spec processor checks for.
///
/// Making this a public function means the processor can report on its own
/// expectations, which is useful for debugging and documentation.
#[must_use]
pub fn expected_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            name: "file_io".into(),
            expected: true,
            stdlib_file: "std/src/io/fs.ash".into(),
        },
        Capability {
            name: "stdio".into(),
            expected: true,
            stdlib_file: "std/src/stdio.ash".into(),
        },
        Capability {
            name: "regex".into(),
            expected: true,
            stdlib_file: "std/src/regex.ash".into(),
        },
        Capability {
            name: "json".into(),
            expected: true,
            stdlib_file: "std/src/json.ash".into(),
        },
        Capability {
            name: "markdown".into(),
            expected: true,
            stdlib_file: "std/src/markdown.ash".into(),
        },
        Capability {
            name: "process".into(),
            expected: true,
            stdlib_file: "std/src/process.ash".into(),
        },
        Capability {
            name: "generic_interfaces".into(),
            expected: false,
            stdlib_file: "std/src/generics.ash".into(),
        },
    ]
}

/// Audit the repository tree against the capability boundary declaration.
///
/// For each capability marked `expected == true`, confirms that the stdlib
/// file exists and contains at least one `pub fn` or `pub type` declaration.
/// Missing or effectively-empty files produce `ToolingGap` warnings.
///
/// Capabilities marked `expected == false` produce informational findings
/// noting that the capability is not yet available.
#[must_use]
pub fn check_capabilities(repo_root: &Path) -> Vec<SpecFinding> {
    let mut findings = Vec::new();

    for cap in expected_capabilities() {
        if cap.expected {
            let path = repo_root.join(&cap.stdlib_file);

            if !path.exists() {
                findings.push(
                    SpecFinding::warning(
                        "ToolingGap",
                        format!(
                            "capability '{}' is expected but stdlib file '{}' is missing",
                            cap.name, cap.stdlib_file
                        ),
                    )
                    .with_file(&cap.stdlib_file),
                );
                continue;
            }

            match fs::read_to_string(&path) {
                Ok(contents) => {
                    if !has_substantive_declarations(&contents) {
                        findings.push(
                            SpecFinding::warning(
                                "ToolingGap",
                                format!(
                                    "capability '{}' stdlib file '{}' exists but contains no \
                                     public fn/type declarations",
                                    cap.name, cap.stdlib_file
                                ),
                            )
                            .with_file(&cap.stdlib_file),
                        );
                    }
                }
                Err(e) => {
                    findings.push(
                        SpecFinding::warning(
                            "ToolingGap",
                            format!(
                                "capability '{}' stdlib file '{}' is unreadable: {e}",
                                cap.name, cap.stdlib_file
                            ),
                        )
                        .with_file(&cap.stdlib_file),
                    );
                }
            }
        } else {
            findings.push(SpecFinding::info(
                "CapabilityPending",
                format!(
                    "capability '{}' is not yet available (pending future phase)",
                    cap.name
                ),
            ));
        }
    }

    findings
}

/// Returns `true` if the file contents contain at least one substantive
/// public declaration (`pub fn`, `pub builtin fn`, or `pub type`), ignoring
/// whitespace and comments.
///
/// Lines starting with `//` or `--` (Ash comments) are skipped entirely to
/// avoid false positives from commented-out declarations.
fn has_substantive_declarations(contents: &str) -> bool {
    contents.lines().any(|line| {
        let trimmed = line.trim();
        // Skip empty lines and comments (Rust `//` and Ash `--`).
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("--") {
            return false;
        }
        // `starts_with` avoids matching `pub fn` inside comments or strings
        // that don't start with `//` (e.g. `/* pub fn ... */`).
        trimmed.starts_with("pub fn")
            || trimmed.starts_with("pub builtin fn")
            || trimmed.starts_with("pub type")
    })
}
