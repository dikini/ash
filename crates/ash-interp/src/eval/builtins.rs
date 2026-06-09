//! Builtin metadata and dispatch-table lookup.

use std::collections::HashMap;

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

        // ── Provider-backed stdlib surfaces intentionally deferred in interp ──
        for (name, arity) in [
            ("http::get", 1),
            ("http::post", 2),
            ("http::put", 2),
            ("http::delete", 1),
            ("time::now", 0),
            ("time::now_iso", 0),
            ("time::epoch_millis", 0),
            ("time::sleep", 1),
            ("io::stdio::read_line", 0),
            ("io::stdio::print", 1),
            ("io::stdio::println", 1),
            ("io::fs::read", 1),
            ("io::fs::read_to_string", 1),
            ("io::fs::write", 2),
            ("io::fs::write_string", 2),
            ("io::fs::append", 2),
            ("io::fs::copy", 2),
            ("io::fs::rename", 2),
            ("io::fs::remove_file", 1),
            ("io::dir::create_dir", 1),
            ("io::dir::create_dir_all", 1),
            ("io::dir::remove_dir", 1),
            ("io::dir::remove_dir_all", 1),
            ("io::dir::read_dir", 1),
            ("io::meta::metadata", 1),
            ("io::meta::is_file", 1),
            ("io::meta::is_dir", 1),
            ("io::meta::len", 1),
            ("io::meta::readonly", 1),
            ("io::buf::read_to_end", 1),
            ("io::buf::read_to_string", 1),
            ("io::buf::write_all", 2),
            ("io::buf::lines", 1),
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
