-- Process execution capability and functions
--
-- Provides parser-checkable runtime-provided function declarations for external
-- process access. The Process capability below records the intended authority
-- contract; concrete capability-wrapper bodies remain deferred until the
-- parser/runtime support a canonical stdlib `act` wrapper spelling.
-- Process execution is effectful (Operational) per the three-pillar principle.

-- Process.run returns a runtime record with stdout, stderr, and exit_code.
-- Process.which returns Some(path) when found and None when absent.
pub capability Process: execute run(cmd: String, args: List<String>) returns Record
                     | execute which(cmd: String) returns Option<String>;

-- Execute a command with arguments, returning the provider output record.
pub builtin fn run(cmd: String, args: List<String>) -> Record;

-- Check if a command exists, returning Some(path) or None.
pub builtin fn which(cmd: String) -> Option<String>;

-- Phase 199 process/channel convenience helper records.
--
-- These helpers describe process/channel plans and expected evidence without
-- spawning processes, opening channels, or bypassing Phase 195 runtime checks.

pub type SpawnJoinPlan = SpawnJoinPlan {
    name: String,
    child_count: Int,
    preserves_sendability: Bool,
    preserves_ownership: Bool,
    propagates_failure: Bool,
};

pub type WorkerPoolPlan = WorkerPoolPlan {
    name: String,
    worker_count: Int,
    queue_bound: Int,
    preserves_sendability: Bool,
};

pub type StreamLoopPlan = StreamLoopPlan {
    name: String,
    stream_name: String,
    capacity: Int,
    closes_stream: Bool,
};

pub type CancellationCleanupPlan = CancellationCleanupPlan {
    name: String,
    cleanup_steps: Int,
    propagates_cancellation: Bool,
};

pub type SendabilityGuard = SendabilityGuard {
    subject: String,
    sendable: Bool,
    owned: Bool,
};

pub type ChannelDiagnosticExpectation = ChannelDiagnosticExpectation {
    stream_name: String,
    state: String,
    structured: Bool,
};

pub type ProcessTraceExpectation = ProcessTraceExpectation {
    event_name: String,
    redacted: Bool,
    retained: Bool,
};

pub fn spawn_join_plan(name: String, child_count: Int) -> SpawnJoinPlan {
    SpawnJoinPlan {
        name: name,
        child_count: child_count,
        preserves_sendability: true,
        preserves_ownership: true,
        propagates_failure: true,
    }
}

pub fn bounded_worker_pool(name: String, worker_count: Int, queue_bound: Int) -> WorkerPoolPlan {
    WorkerPoolPlan {
        name: name,
        worker_count: worker_count,
        queue_bound: queue_bound,
        preserves_sendability: true,
    }
}

pub fn channel_loop_plan(name: String, stream_name: String, capacity: Int) -> StreamLoopPlan {
    StreamLoopPlan {
        name: name,
        stream_name: stream_name,
        capacity: capacity,
        closes_stream: true,
    }
}

pub fn cancellation_cleanup(name: String, cleanup_steps: Int) -> CancellationCleanupPlan {
    CancellationCleanupPlan {
        name: name,
        cleanup_steps: cleanup_steps,
        propagates_cancellation: true,
    }
}

pub fn sendability_guard(subject: String, sendable: Bool, owned: Bool) -> SendabilityGuard {
    SendabilityGuard { subject: subject, sendable: sendable, owned: owned }
}

pub fn channel_diagnostic(stream_name: String, state: String) -> ChannelDiagnosticExpectation {
    ChannelDiagnosticExpectation { stream_name: stream_name, state: state, structured: true }
}

pub fn process_trace(event_name: String, redacted: Bool) -> ProcessTraceExpectation {
    ProcessTraceExpectation { event_name: event_name, redacted: redacted, retained: true }
}
