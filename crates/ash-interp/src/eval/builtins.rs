//! Builtin metadata and dispatch-table lookup.

use ash_core::Effect;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinEntry {
    /// Number of required parameters (0 for variadic builtins).
    pub arity: usize,
    /// Whether the builtin accepts a variable number of arguments.
    pub variadic: bool,
    /// Whether the runtime implementation is present.
    /// Entries with `implemented: false` produce [`crate::error::EvalError::UnimplementedBuiltin`].
    pub implemented: bool,
}

/// Trusted runtime hook metadata for a host-facing builtin.
///
/// This metadata describes the authority, sandbox, and provenance requirements for a host hook. It
/// is descriptive only; it does not grant authority by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinHostHookMetadata {
    /// Fully qualified builtin dispatch name.
    pub builtin_name: &'static str,
    /// Stable operation identity used for row/admission checks.
    pub operation_identity: &'static str,
    /// Effect level for this operation.
    pub effect: Effect,
    /// Required operation/provider rows.
    pub required_rows: &'static [&'static str],
    /// Sandbox policy identity that must be checked before host execution.
    pub sandbox_policy: &'static str,
    /// Provenance policy identity used for host-boundary evidence.
    pub provenance_policy: &'static str,
    /// Whether this metadata grants authority.
    ///
    /// This must remain false. Authority comes from admitted rows/providers/resources, not from a
    /// builtin table entry.
    pub grants_authority: bool,
}

/// Builtin host-hook metadata validation errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BuiltinHostHookMetadataError {
    /// An implemented host-facing builtin has no metadata.
    #[error("builtin '{builtin_name}' is missing host hook metadata")]
    MissingHostHookMetadata {
        /// Fully qualified builtin name.
        builtin_name: String,
    },
    /// Metadata names a different builtin than the dispatch entry being checked.
    #[error("builtin '{builtin_name}' has host hook metadata for '{metadata_builtin_name}'")]
    MismatchedBuiltinName {
        /// Fully qualified builtin name from the dispatch table.
        builtin_name: String,
        /// Builtin name embedded in the metadata record.
        metadata_builtin_name: String,
    },
    /// Metadata attempted to grant authority directly.
    #[error("builtin '{builtin_name}' host hook metadata must not grant authority")]
    MetadataGrantsAuthority {
        /// Fully qualified builtin name.
        builtin_name: String,
    },
}

/// Returns the builtin dispatch table mapping qualified names to entries.
///
/// Qualified names use `"module::func"` format (e.g., `"string::concat"`).
/// Unqualified names are used for builtins that accept any module prefix
/// (e.g., `"len"`, `"head"`).
pub fn builtin_dispatch_table() -> &'static HashMap<&'static str, BuiltinEntry> {
    static TABLE: std::sync::OnceLock<HashMap<&'static str, BuiltinEntry>> =
        std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut m = HashMap::new();
        // ── String module builtins (qualified) ──
        m.insert(
            "string::concat",
            BuiltinEntry {
                arity: 0,
                variadic: true,
                implemented: true,
            },
        );
        m.insert(
            "string::starts_with",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "string::ends_with",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "string::is_empty",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Regex module builtins (qualified) ──
        m.insert(
            "regex::find",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "regex::matches",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "regex::replace",
            BuiltinEntry {
                arity: 3,
                variadic: false,
                implemented: true,
            },
        );

        // ── String case / whitespace builtins ──
        m.insert(
            "string::to_upper",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "string::to_lower",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "string::trim",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Provider-backed stdlib surfaces ──
        for (name, arity) in [
            ("http::get", 1),
            ("http::post", 2),
            ("http::put", 2),
            ("http::delete", 1),
            ("fs::exists", 1),
            ("fs::read_to_string", 1),
            ("fs::append", 2),
            ("fs::write_string", 2),
            ("dir::read_dir", 1),
            ("meta::metadata", 1),
            ("io::fs::exists", 1),
            ("io::fs::read_to_string", 1),
            ("io::fs::append", 2),
            ("io::fs::write_string", 2),
            ("io::dir::read_dir", 1),
            ("io::meta::metadata", 1),
            ("time::now", 0),
            ("time::now_iso", 0),
            ("time::epoch_millis", 0),
            ("time::sleep", 1),
            ("logging::debug", 1),
            ("logging::info", 1),
            ("logging::warn", 1),
            ("logging::error", 1),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic: false,
                    implemented: true,
                },
            );
        }

        // ── Provider-backed stdlib surfaces intentionally deferred in interp ──
        for (name, arity) in [
            ("io::stdio::read_line", 0),
            ("io::stdio::print", 1),
            ("io::stdio::println", 1),
            ("io::fs::read", 1),
            ("io::fs::write", 2),
            ("io::fs::copy", 2),
            ("io::fs::rename", 2),
            ("io::fs::remove_file", 1),
            ("io::dir::create_dir", 1),
            ("io::dir::create_dir_all", 1),
            ("io::dir::remove_dir", 1),
            ("io::dir::remove_dir_all", 1),
            ("io::meta::is_file", 1),
            ("io::meta::is_dir", 1),
            ("io::meta::len", 1),
            ("io::meta::readonly", 1),
            ("io::buf::read_to_end", 1),
            ("io::buf::read_to_string", 1),
            ("io::buf::write_all", 2),
            ("io::buf::lines", 1),
            ("llm::dispatch::complete", 4),
            ("llm::dispatch::complete_with_tools", 5),
            ("llm::dispatch::stream", 4),
            ("llm::dispatch::embed", 3),
            ("llm::dispatch::list_models", 1),
            ("test::quickcheck::bool::gen", 1),
            ("test::quickcheck::bool::shrink", 1),
            ("test::quickcheck::context::seed", 1),
            ("test::quickcheck::context::size", 1),
            ("test::quickcheck::context::split", 2),
            ("test::quickcheck::context::variant", 2),
            ("test::quickcheck::context::indexed", 3),
            ("test::quickcheck::context::resize", 2),
            ("test::quickcheck::context::choose_int", 3),
            ("test::quickcheck::context::choose_bool", 1),
            ("test::quickcheck::int::gen", 1),
            ("test::quickcheck::int::gen_small", 1),
            ("test::quickcheck::int::gen_positive", 1),
            ("test::quickcheck::int::gen_nonzero", 1),
            ("test::quickcheck::int::shrink", 1),
            ("test::quickcheck::list::gen", 1),
            ("test::quickcheck::list::gen_nonempty_int", 1),
            ("test::quickcheck::list::gen_sorted_int", 1),
            ("test::quickcheck::list::shrink", 1),
            ("test::quickcheck::list::shrink_int_list", 1),
            ("test::quickcheck::strategy::no_shrink", 1),
            ("test::quickcheck::string::gen", 1),
            ("test::quickcheck::string::gen_identifier", 1),
            ("test::quickcheck::string::shrink", 1),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic: false,
                    implemented: false,
                },
            );
        }

        // ── Process module builtins (qualified) ──
        m.insert(
            "process::run",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "process::which",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Act module bridge builtins (qualified) ──
        m.insert(
            "act::unit",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "act::bind",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "act::__guard",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "act::policy_check",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Proc module bridge builtins (qualified) ──
        for (name, arity) in [
            ("proc::unit", 1),
            ("proc::from_act", 1),
            ("proc::bind", 2),
            ("proc::then", 2),
            ("proc::await", 1),
            ("proc::yield", 0),
            ("proc::par", 2),
            ("proc::scatter", 2),
            ("proc::join", 2),
            ("proc::gather", 1),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic: false,
                    implemented: true,
                },
            );
        }

        // ── Workflow module bridge builtins (qualified) ──
        for (name, arity) in [
            ("workflow::unit", 1),
            ("workflow::from_act", 1),
            ("workflow::from_proc", 1),
            ("workflow::bind", 2),
            ("workflow::then", 2),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic: false,
                    implemented: true,
                },
            );
        }

        // ── List module builtins (qualified) ──
        m.insert(
            "list::len",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::head",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::tail",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::append",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::concat",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::filter",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "list::map",
            BuiltinEntry {
                arity: 2,
                variadic: false,
                implemented: true,
            },
        );

        // ── Record module builtins (qualified) ──
        for (name, arity, variadic) in [
            ("record::keys", 1, false),
            ("record::values", 1, false),
            ("record::record", 0, true),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic,
                    implemented: true,
                },
            );
        }

        // ── Unqualified builtins ──
        let unqualified = [
            ("len", 1, false),
            ("head", 1, false),
            ("tail", 1, false),
            ("append", 2, false),
            ("concat", 2, false),
            ("filter", 2, false),
            ("map", 2, false),
            ("starts_with", 2, false),
            ("ends_with", 2, false),
            ("keys", 1, false),
            ("values", 1, false),
            ("is_int", 1, false),
            ("is_string", 1, false),
            ("is_bool", 1, false),
            ("is_list", 1, false),
            ("is_record", 1, false),
            ("is_null", 1, false),
            ("has_evidence", 1, false),
            ("is_redacted", 1, false),
            ("is_authority_neutral", 1, false),
            ("provider_outcome_is_success", 1, false),
            ("provider_outcome_is_denied", 1, false),
            ("provider_outcome_is_failure", 1, false),
        ];
        for (name, arity, variadic) in unqualified {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic,
                    implemented: true,
                },
            );
        }

        // ── Evidence helper builtins (qualified) ──
        for name in [
            "evidence::has_evidence",
            "evidence::is_redacted",
            "evidence::is_authority_neutral",
            "evidence::provider_outcome_is_success",
            "evidence::provider_outcome_is_denied",
            "evidence::provider_outcome_is_failure",
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity: 1,
                    variadic: false,
                    implemented: true,
                },
            );
        }

        // ── Predicate module builtins (qualified) ──
        for (name, arity, variadic) in [
            ("predicate::is_int", 1, false),
            ("predicate::is_string", 1, false),
            ("predicate::is_bool", 1, false),
            ("predicate::is_list", 1, false),
            ("predicate::is_record", 1, false),
            ("predicate::is_null", 1, false),
        ] {
            m.insert(
                name,
                BuiltinEntry {
                    arity,
                    variadic,
                    implemented: true,
                },
            );
        }

        // ── JSON module builtins (qualified) ──
        m.insert(
            "json::parse",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "json::stringify",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );
        m.insert(
            "json::stringify_pretty",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        // ── Markdown module builtins (qualified) ──
        m.insert(
            "markdown::parse",
            BuiltinEntry {
                arity: 1,
                variadic: false,
                implemented: true,
            },
        );

        m.insert(
            "record",
            BuiltinEntry {
                arity: 0,
                variadic: true,
                implemented: true,
            },
        );
        m
    })
}

/// Returns true if the builtin name is host-facing and must be backed by host-hook metadata when
/// implemented.
pub fn builtin_requires_host_hook_metadata(name: &str, _entry: &BuiltinEntry) -> bool {
    name.starts_with("process::")
        || name.starts_with("http::")
        || name.starts_with("time::")
        || name.starts_with("io::")
        || name.starts_with("mcp::")
        || name.starts_with("llm::")
}

/// Returns trusted host-hook metadata for an implemented host-facing builtin.
pub fn builtin_host_hook_metadata(name: &str) -> Option<&'static BuiltinHostHookMetadata> {
    static PROCESS_RUN_ROWS: &[&str] = &["process.run"];
    static PROCESS_WHICH_ROWS: &[&str] = &["process.which"];
    static HTTP_GET_ROWS: &[&str] = &["http.get"];
    static HTTP_POST_ROWS: &[&str] = &["http.post"];
    static HTTP_PUT_ROWS: &[&str] = &["http.put"];
    static HTTP_DELETE_ROWS: &[&str] = &["http.delete"];
    static TIME_NOW_ROWS: &[&str] = &["time.now"];
    static TIME_NOW_ISO_ROWS: &[&str] = &["time.now_iso"];
    static TIME_EPOCH_MILLIS_ROWS: &[&str] = &["time.epoch_millis"];
    static TIME_SLEEP_ROWS: &[&str] = &["time.sleep"];
    static LOGGING_DEBUG_ROWS: &[&str] = &["logging.debug"];
    static LOGGING_INFO_ROWS: &[&str] = &["logging.info"];
    static LOGGING_WARN_ROWS: &[&str] = &["logging.warn"];
    static LOGGING_ERROR_ROWS: &[&str] = &["logging.error"];
    static PROCESS_RUN: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "process::run",
        operation_identity: "process.run",
        effect: Effect::Operational,
        required_rows: PROCESS_RUN_ROWS,
        sandbox_policy: "process-command",
        provenance_policy: "host.process.run",
        grants_authority: false,
    };
    static PROCESS_WHICH: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "process::which",
        operation_identity: "process.which",
        effect: Effect::Epistemic,
        required_rows: PROCESS_WHICH_ROWS,
        sandbox_policy: "process-command-lookup",
        provenance_policy: "host.process.which",
        grants_authority: false,
    };
    static HTTP_GET: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "http::get",
        operation_identity: "http.get",
        effect: Effect::Epistemic,
        required_rows: HTTP_GET_ROWS,
        sandbox_policy: "host.http.get",
        provenance_policy: "host.http.get.redacted",
        grants_authority: false,
    };
    static HTTP_POST: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "http::post",
        operation_identity: "http.post",
        effect: Effect::Operational,
        required_rows: HTTP_POST_ROWS,
        sandbox_policy: "host.http.post",
        provenance_policy: "host.http.post.redacted",
        grants_authority: false,
    };
    static HTTP_PUT: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "http::put",
        operation_identity: "http.put",
        effect: Effect::Operational,
        required_rows: HTTP_PUT_ROWS,
        sandbox_policy: "host.http.put",
        provenance_policy: "host.http.put.redacted",
        grants_authority: false,
    };
    static HTTP_DELETE: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "http::delete",
        operation_identity: "http.delete",
        effect: Effect::Operational,
        required_rows: HTTP_DELETE_ROWS,
        sandbox_policy: "host.http.delete",
        provenance_policy: "host.http.delete.redacted",
        grants_authority: false,
    };
    static TIME_NOW: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "time::now",
        operation_identity: "time.now",
        effect: Effect::Epistemic,
        required_rows: TIME_NOW_ROWS,
        sandbox_policy: "host.time.now",
        provenance_policy: "host.time.now.redacted",
        grants_authority: false,
    };
    static TIME_NOW_ISO: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "time::now_iso",
        operation_identity: "time.now_iso",
        effect: Effect::Epistemic,
        required_rows: TIME_NOW_ISO_ROWS,
        sandbox_policy: "host.time.now",
        provenance_policy: "host.time.now.redacted",
        grants_authority: false,
    };
    static TIME_EPOCH_MILLIS: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "time::epoch_millis",
        operation_identity: "time.epoch_millis",
        effect: Effect::Epistemic,
        required_rows: TIME_EPOCH_MILLIS_ROWS,
        sandbox_policy: "host.time.now",
        provenance_policy: "host.time.now.redacted",
        grants_authority: false,
    };
    static TIME_SLEEP: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "time::sleep",
        operation_identity: "time.sleep",
        effect: Effect::Operational,
        required_rows: TIME_SLEEP_ROWS,
        sandbox_policy: "host.time.sleep",
        provenance_policy: "host.time.sleep.redacted",
        grants_authority: false,
    };
    static LOGGING_DEBUG: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "logging::debug",
        operation_identity: "logging.debug",
        effect: Effect::Operational,
        required_rows: LOGGING_DEBUG_ROWS,
        sandbox_policy: "host.logging.write",
        provenance_policy: "host.logging.write.redacted",
        grants_authority: false,
    };
    static LOGGING_INFO: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "logging::info",
        operation_identity: "logging.info",
        effect: Effect::Operational,
        required_rows: LOGGING_INFO_ROWS,
        sandbox_policy: "host.logging.write",
        provenance_policy: "host.logging.write.redacted",
        grants_authority: false,
    };
    static LOGGING_WARN: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "logging::warn",
        operation_identity: "logging.warn",
        effect: Effect::Operational,
        required_rows: LOGGING_WARN_ROWS,
        sandbox_policy: "host.logging.write",
        provenance_policy: "host.logging.write.redacted",
        grants_authority: false,
    };
    static LOGGING_ERROR: BuiltinHostHookMetadata = BuiltinHostHookMetadata {
        builtin_name: "logging::error",
        operation_identity: "logging.error",
        effect: Effect::Operational,
        required_rows: LOGGING_ERROR_ROWS,
        sandbox_policy: "host.logging.write",
        provenance_policy: "host.logging.write.redacted",
        grants_authority: false,
    };

    match name {
        "process::run" => Some(&PROCESS_RUN),
        "process::which" => Some(&PROCESS_WHICH),
        "http::get" => Some(&HTTP_GET),
        "http::post" => Some(&HTTP_POST),
        "http::put" => Some(&HTTP_PUT),
        "http::delete" => Some(&HTTP_DELETE),
        "time::now" => Some(&TIME_NOW),
        "time::now_iso" => Some(&TIME_NOW_ISO),
        "time::epoch_millis" => Some(&TIME_EPOCH_MILLIS),
        "time::sleep" => Some(&TIME_SLEEP),
        "logging::debug" => Some(&LOGGING_DEBUG),
        "logging::info" => Some(&LOGGING_INFO),
        "logging::warn" => Some(&LOGGING_WARN),
        "logging::error" => Some(&LOGGING_ERROR),
        _ => None,
    }
}

/// Validate host-hook metadata for one builtin dispatch entry.
///
/// Forward-declared but unimplemented host-facing builtins are allowed to exist without metadata so
/// they continue to fail at the unimplemented-builtin execution gate.
pub fn validate_builtin_host_hook_metadata(
    builtin_name: &str,
    entry: &BuiltinEntry,
    metadata: Option<&BuiltinHostHookMetadata>,
) -> Result<(), BuiltinHostHookMetadataError> {
    if !builtin_requires_host_hook_metadata(builtin_name, entry) || !entry.implemented {
        return Ok(());
    }

    let metadata =
        metadata.ok_or_else(|| BuiltinHostHookMetadataError::MissingHostHookMetadata {
            builtin_name: builtin_name.to_string(),
        })?;

    if metadata.builtin_name != builtin_name {
        return Err(BuiltinHostHookMetadataError::MismatchedBuiltinName {
            builtin_name: builtin_name.to_string(),
            metadata_builtin_name: metadata.builtin_name.to_string(),
        });
    }

    if metadata.grants_authority {
        return Err(BuiltinHostHookMetadataError::MetadataGrantsAuthority {
            builtin_name: builtin_name.to_string(),
        });
    }

    Ok(())
}

/// Check whether `(func, module)` identifies a known builtin.
///
/// Looks up both the qualified form `"module::func"` (when `module` is `Some`)
/// and the bare `func` name in the dispatch table. O(1) via HashMap lookups.
pub fn is_known_builtin(func: &str, module: Option<&str>) -> bool {
    let table = builtin_dispatch_table();

    // Try qualified name first (O(1))
    if let Some(mod_name) = module {
        let qualified = format!("{mod_name}::{func}");
        if table.contains_key(qualified.as_str()) {
            return true;
        }
    }

    // Try unqualified name (O(1))
    table.contains_key(func)
}
