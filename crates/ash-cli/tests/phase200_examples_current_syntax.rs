use std::path::{Path, PathBuf};

const PRODUCTIVE_ROOTS: &[&str] = &[
    "examples/10-testing-helpers",
    "examples/11-process-channel-helpers",
];

const SCAN_ROOTS: &[&str] = &["examples"];

const PATTERNS: &[(&str, &str)] = &[
    ("observe-with", "observe-with"),
    ("act-with", "act-with"),
    ("proc-carrier", "Proc-carrier"),
    ("act-carrier", "Act-carrier"),
    ("application-carrier", "Application-carrier"),
    ("removed-application", "removed-application"),
    ("template-application", "application-template"),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_source_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("ash" | "md")
        ) {
            files.push(path.to_path_buf());
        }
        return;
    }

    let entries =
        std::fs::read_dir(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    for entry in entries {
        let entry = entry.expect("read dir entry");
        collect_source_files(&entry.path(), files);
    }
}

fn contains_pattern(line: &str, pattern: &str, needle: &str) -> bool {
    match pattern {
        "observe-with" => contains_token_followed_by_with(line, "observe"),
        "act-with" => contains_token_followed_by_with(line, "act"),
        "proc-carrier" => line.contains(&["Pr", "oc<"].concat()),
        "act-carrier" => line.contains(&["A", "ct<"].concat()),
        "application-carrier" => line.contains(&["Work", "flow<"].concat()),
        "removed-application" => line.contains("removed application"),
        "template-application" => line.contains("application template"),
        _ => line.contains(needle),
    }
}

fn contains_token_followed_by_with(line: &str, token: &str) -> bool {
    let mut saw_token = false;
    for part in line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')) {
        if part.is_empty() {
            continue;
        }
        if saw_token && part == "with" {
            return true;
        }
        saw_token |= part == token;
    }
    false
}

fn has_historical_marker(path: &Path, source: &str) -> bool {
    if source.contains("REFERENCE-ONLY")
        || source.contains("reference-only")
        || source.contains("historical")
        || source.contains("compatibility")
    {
        return true;
    }

    for ancestor in path.ancestors().skip(1) {
        let readme = ancestor.join("README.md");
        if let Ok(readme_source) = std::fs::read_to_string(readme) {
            let lower = readme_source.to_ascii_lowercase();
            if lower.contains("reference-only")
                || lower.contains("historical")
                || lower.contains("compatibility")
            {
                return true;
            }
        }
    }

    false
}

#[test]
fn productive_example_roots_do_not_contain_removed_forms() {
    let root = repo_root();
    let mut findings = Vec::new();

    for relative_root in PRODUCTIVE_ROOTS {
        for path in source_files(&root.join(relative_root)) {
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            for (pattern, needle) in PATTERNS {
                if source
                    .lines()
                    .any(|line| contains_pattern(line, pattern, needle))
                {
                    findings.push(format!("{}:{pattern}", relative(&path)));
                }
            }
        }
    }

    assert!(
        findings.is_empty(),
        "productive examples must stay current-syntax only: {findings:#?}"
    );
}

#[test]
fn retained_historical_example_hits_are_marked_reference_or_compatibility() {
    let root = repo_root();
    let mut findings = Vec::new();

    for relative_root in SCAN_ROOTS {
        for path in source_files(&root.join(relative_root)) {
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            let has_hit = PATTERNS.iter().any(|(pattern, needle)| {
                source
                    .lines()
                    .any(|line| contains_pattern(line, pattern, needle))
            });
            if has_hit && !has_historical_marker(&path, &source) {
                findings.push(relative(&path));
            }
        }
    }

    assert!(
        findings.is_empty(),
        "retained historical examples must be visibly historical/reference-only/compatibility: {findings:#?}"
    );
}
