//! Test metadata: parse file-level `@test` blocks from Ash test files.
//!
//! TASK-512: Authored test metadata and execution model.

use std::path::Path;

use crate::test_runner::quickcheck::QuickCheckStrategyOverride;

/// Parsed test metadata from a file-level comment block.
#[derive(Debug, Clone, Default)]
pub struct TestMetadata {
    /// Test name (defaults to file stem).
    pub name: Option<String>,
    /// Test kind (unit, integration, e2e, property, smallworld).
    pub kind: Option<String>,
    /// Tags for filtering.
    pub tags: Vec<String>,
    /// Timeout in milliseconds (0 = no timeout).
    pub timeout_ms: u64,
    /// Capabilities needed for this test.
    pub capabilities: Vec<String>,
    /// Seed for property tests.
    pub seed: Option<u64>,
    /// Max cases for property tests.
    pub max_cases: Option<usize>,
    /// Max worlds for small-world tests.
    pub max_worlds: Option<usize>,
    /// Generated property parameter declarations, e.g. `x: Int, xs: List<Int>`.
    pub generated_params: Vec<String>,
    /// Simple authored property oracle evaluated for each generated binding.
    pub property: Option<String>,
    /// QuickCheck-style explicit strategy overrides keyed by binding name.
    pub quickcheck_strategies: Vec<QuickCheckStrategyOverride>,
    /// Whether ordinary QuickCheck `Arbitrary<A>` evidence was explicitly
    /// imported into the source scope.
    pub quickcheck_arbitrary_evidence_in_scope: bool,
    /// Whether this test is expected to fail.
    pub xfail: bool,
    /// Quarantine reason, when this test is quarantined.
    pub quarantine: Option<String>,
    /// Whether a quarantine directive was present but malformed.
    pub quarantine_malformed: bool,
    /// Deterministic fixture hook: fail attempts before this one.
    pub flaky_until_attempt: Option<usize>,
}

impl TestMetadata {
    /// Parse test metadata from the first comment block of an Ash source file.
    ///
    /// Looks for lines matching `// @test key: value` or `// @test key` patterns.
    /// Returns default metadata if no `@test` annotations are found.
    pub fn parse_from_source(source: &str) -> Self {
        let mut meta = Self::default();
        let mut in_test_block = false;

        for line in source.lines() {
            let trimmed = line.trim();

            let comment_body = trimmed
                .strip_prefix("//")
                .or_else(|| trimmed.strip_prefix("--"))
                .map(str::trim);

            if let Some(rest) = comment_body {
                if rest.starts_with("@test") {
                    in_test_block = true;
                    let directive = rest.strip_prefix("@test").unwrap().trim();
                    if !directive.is_empty() {
                        parse_directive(directive, &mut meta);
                    }
                    continue;
                }
                if in_test_block {
                    if rest.starts_with('@') {
                        in_test_block = false;
                        continue;
                    }
                    if rest.starts_with("@test") {
                        let directive = rest.strip_prefix("@test").unwrap().trim();
                        if !directive.is_empty() {
                            parse_directive(directive, &mut meta);
                        }
                        continue;
                    }
                    continue;
                }
            } else if !trimmed.is_empty() {
                break;
            }
        }

        meta.quickcheck_arbitrary_evidence_in_scope = source_imports_quickcheck_arbitrary(source);
        meta
    }

    /// Parse metadata from a file.
    pub fn parse_from_file(path: &Path) -> std::io::Result<Self> {
        let source = std::fs::read_to_string(path)?;
        Ok(Self::parse_from_source(&source))
    }

    /// Get the effective test name, falling back to file stem.
    pub fn effective_name(&self, path: &Path) -> String {
        self.name.clone().unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        })
    }

    /// Return the explicit quickcheck strategy path for a generated binding.
    pub fn quickcheck_strategy_for(&self, binding: &str) -> Option<&str> {
        self.quickcheck_strategies
            .iter()
            .find(|strategy| strategy.binding == binding)
            .map(|strategy| strategy.strategy_path.as_str())
    }
}

fn parse_directive(directive: &str, meta: &mut TestMetadata) {
    let parts: Vec<&str> = directive.splitn(2, ':').collect();
    let key = parts[0].trim();
    let value = parts.get(1).map(|v| v.trim()).unwrap_or("");

    if let Some(binding) = key.strip_prefix("strategy ") {
        if !binding.trim().is_empty() && !value.is_empty() {
            meta.quickcheck_strategies.push(QuickCheckStrategyOverride {
                binding: binding.trim().to_string(),
                strategy_path: value.to_string(),
            });
        }
        return;
    }

    match key {
        "name" => {
            if !value.is_empty() {
                meta.name = Some(value.to_string());
            }
        }
        "kind" => {
            if !value.is_empty() {
                meta.kind = Some(value.to_string());
            }
        }
        "tags" => {
            if !value.is_empty() {
                meta.tags = value
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
            }
        }
        "timeout_ms" => {
            if let Ok(ms) = value.parse() {
                meta.timeout_ms = ms;
            }
        }
        "capabilities" => {
            if !value.is_empty() {
                meta.capabilities = value
                    .split(',')
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty())
                    .collect();
            }
        }
        "seed" => {
            if let Ok(seed) = value.parse() {
                meta.seed = Some(seed);
            }
        }
        "max_cases" => {
            if let Ok(n) = value.parse() {
                meta.max_cases = Some(n);
            }
        }
        "max_worlds" => {
            if let Ok(n) = value.parse() {
                meta.max_worlds = Some(n);
            }
        }
        "params" => {
            if !value.is_empty() {
                meta.generated_params = split_param_list(value);
            }
        }
        "property" => {
            if !value.is_empty() {
                meta.property = Some(value.to_string());
            }
        }
        "quickcheck" => {
            meta.kind = Some("property".to_string());
        }
        "xfail" => {
            meta.xfail = true;
        }
        "quarantine" => {
            if value.is_empty() {
                meta.quarantine_malformed = true;
            } else {
                meta.quarantine = Some(value.to_string());
            }
        }
        "flaky_until_attempt" => match value.parse::<usize>() {
            Ok(attempt) if attempt > 0 => meta.flaky_until_attempt = Some(attempt),
            _ => {}
        },
        _ => {} // Unknown directive, ignore
    }
}

fn split_param_list(value: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in value.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let param = value[start..index].trim();
                if !param.is_empty() {
                    params.push(param.to_string());
                }
                start = index + 1;
            }
            _ => {}
        }
    }
    let param = value[start..].trim();
    if !param.is_empty() {
        params.push(param.to_string());
    }
    params
}

fn source_imports_quickcheck_arbitrary(source: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim().trim_end_matches(';').trim();
        let Some(rest) = trimmed.strip_prefix("use ").map(str::trim) else {
            return false;
        };

        rest == "test::quickcheck::prelude"
            || rest == "test::quickcheck::Arbitrary"
            || rest
                .strip_prefix("test::quickcheck::{")
                .and_then(|items| items.strip_suffix('}'))
                .is_some_and(|items| {
                    items
                        .split(',')
                        .map(str::trim)
                        .any(|item| item == "Arbitrary" || item == "prelude")
                })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_source() {
        let meta = TestMetadata::parse_from_source("fn main() { 1 }");
        assert!(meta.name.is_none());
        assert!(meta.tags.is_empty());
    }

    #[test]
    fn parse_name_directive() {
        let source = "// @test name: my_test\n// rest\nfn main() {}";
        let meta = TestMetadata::parse_from_source(source);
        assert_eq!(meta.name.as_deref(), Some("my_test"));
    }

    #[test]
    fn parse_quickcheck_arbitrary_evidence_imports() {
        let source = r#"
-- @test name: imported_quickcheck
-- @test kind: property
-- @test params: b: Bool
use test::quickcheck::{Arbitrary};
fn main() -> Bool { true }
"#;
        let meta = TestMetadata::parse_from_source(source);
        assert!(meta.quickcheck_arbitrary_evidence_in_scope);

        let no_import =
            TestMetadata::parse_from_source("-- @test params: b: Bool\nfn main() -> Bool { true }");
        assert!(!no_import.quickcheck_arbitrary_evidence_in_scope);
    }

    #[test]
    fn parse_name_directive_from_ash_comments() {
        let source = "-- @test name: my_test\n-- rest\nfn main() {}";
        let meta = TestMetadata::parse_from_source(source);
        assert_eq!(meta.name.as_deref(), Some("my_test"));
    }

    #[test]
    fn parse_multiple_directives() {
        let source = "// @test name: foo\n// @test kind: integration\n// @test tags: slow, io\n// @test timeout_ms: 5000\n";
        let meta = TestMetadata::parse_from_source(source);
        assert_eq!(meta.name.as_deref(), Some("foo"));
        assert_eq!(meta.kind.as_deref(), Some("integration"));
        assert_eq!(meta.tags, vec!["slow", "io"]);
        assert_eq!(meta.timeout_ms, 5000);
    }

    #[test]
    fn parse_xfail() {
        let source = "// @test xfail\n";
        let meta = TestMetadata::parse_from_source(source);
        assert!(meta.xfail);
    }

    #[test]
    fn parse_seed_and_max_cases() {
        let source = "// @test seed: 42\n// @test max_cases: 100\n";
        let meta = TestMetadata::parse_from_source(source);
        assert_eq!(meta.seed, Some(42));
        assert_eq!(meta.max_cases, Some(100));
    }

    #[test]
    fn parse_generated_property_directives() {
        let source = "// @test kind: property\n// @test params: x: Int, xs: List<Int>\n// @test property: x == x\n";
        let meta = TestMetadata::parse_from_source(source);
        assert_eq!(meta.generated_params, vec!["x: Int", "xs: List<Int>"]);
        assert_eq!(meta.property.as_deref(), Some("x == x"));
    }

    #[test]
    fn parse_quickcheck_strategy_override() {
        let source =
            "-- @test kind: property\n-- @test strategy xs: test::quickcheck::sorted_int_lists\n";
        let meta = TestMetadata::parse_from_source(source);
        assert_eq!(
            meta.quickcheck_strategy_for("xs"),
            Some("test::quickcheck::sorted_int_lists")
        );
    }

    #[test]
    fn effective_name_fallback() {
        let meta = TestMetadata::default();
        let path = Path::new("tests/ash/unit/my_test.ash");
        assert_eq!(meta.effective_name(path), "my_test");
    }
}
