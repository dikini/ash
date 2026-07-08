-- Ash Standard Library
--
-- This module provides the core types and functions for target Ash.

-- Core types
pub use option::{Option, Some, None};
pub use result::{Result, Ok, Err};
pub use runtime::{RuntimeError, Args};
pub use runtime::supervisor::{system_supervisor};

-- Test library (Phase 76)
pub use test::{
    assert_true,
    assert_false,
    assert_eq_int,
    assert_ne_int,
    assert_eq_string,
    assert_eq_bool,
    fail_test,
    AssertionEvidence,
    PropertyEvidence,
    LawEvidence,
    Counterexample,
    TestCoverageArtifact,
    TestMutationArtifact,
    FlakeQuarantine,
    ProviderEvidenceSummary,
    DeterministicProviderProfileFixture,
    CommonTestCase,
    assertion_evidence,
    assert_named,
    property_evidence,
    law_evidence,
    counterexample,
    coverage_evidence,
    mutation_evidence,
    flake_quarantine,
    provider_evidence_summary,
    deterministic_profile_fixture,
    test_clock_fixture,
    common_test_case,
};

-- LLM types
pub mod llm;
pub use llm::{
    Role,
    Message, ToolCall, ToolCallDelta, ToolDef,
    Usage, ChatResponse, Embedding, ChatChunk,
    system, user, assistant, assistant_with_tools, tool_result, message,
    is_system, is_user, is_assistant, is_tool, sender, content,
    get_tool_calls, has_tool_calls, get_tool_call_id,
    render_plaintext, render_markdown,
    count, last, filter_user, filter_assistant, append, prepend
};

-- IO types
pub use io::{Error, ErrorKind};
pub use io::path::{PathBuf, from_string, join, parent, file_name, extension, is_absolute};
pub use io::stdio::{read_line, print, println};
pub use io::fs::{read, read_to_string, write, write_string, append, copy, rename, remove_file};
pub use io::dir::{create_dir, create_dir_all, remove_dir, remove_dir_all, read_dir};
pub use io::meta::{Metadata, metadata, is_file, is_dir, len, readonly};
pub use io::buf::{read_to_end, read_to_string, write_all, lines};

-- Logging provider helpers
pub use logging::{debug, info, warn, error};

-- Provider/profile evidence helpers
pub use evidence::{
    has_evidence,
    is_redacted,
    is_authority_neutral,
    provider_outcome_is_success,
    provider_outcome_is_denied,
    provider_outcome_is_failure
};

-- Regex functions
pub use regex::{find, matches, replace};

-- Process functions
pub use process::{
    run,
    which,
    SpawnJoinPlan,
    WorkerPoolPlan,
    StreamLoopPlan,
    CancellationCleanupPlan,
    SendabilityGuard,
    ChannelDiagnosticExpectation,
    ProcessTraceExpectation,
    spawn_join_plan,
    bounded_worker_pool,
    channel_loop_plan,
    cancellation_cleanup,
    sendability_guard,
    channel_diagnostic,
    process_trace,
};

-- Standard algebra interfaces
pub mod algebra;

-- JSON functions
pub use json::{parse, stringify, stringify_pretty};

-- Markdown functions
pub use markdown::{parse};

-- Helper functions
pub use option::{
    is_some,
    is_none,
    unwrap as unwrap_opt,
    unwrap_or as unwrap_or_opt,
    map as map_opt,
    and_opt,
    or_opt,
    ok_or
};

pub use result::{
    is_ok,
    is_err,
    unwrap as unwrap_res,
    unwrap_err,
    unwrap_or as unwrap_or_res,
    map as map_res,
    map_err,
    and_then,
    ok,
    err as err_opt
};
