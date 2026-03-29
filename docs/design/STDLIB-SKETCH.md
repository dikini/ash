# Ash Standard Library Sketch

**Date**: 2026-03-28  
**Status**: Design sketch based on capability survey and FFI design  
**Syntax**: SPEC-002 compliant

---

## Overview

The Ash standard library provides base capabilities implemented in Rust via the FFI bridge. Each capability is declared in Ash using SPEC-002 syntax and backed by a Rust implementation.

**Key principles**:
- **Unique names**: No two capabilities share the same fully-qualified name in a scope
- **Effect classification**: Each capability declares its effect type (`read`, `write`, `external`, etc.)
- **Type bridging**: Ash types (`Int`, `Float`, `String`, etc.) bridge to Rust types via the `AshBridge` trait
- **No generics exposed**: LLM-facing interfaces use concrete types only

---

## Module Structure

```
std/
├── io.ash           # File system operations
├── net.ash          # Network operations  
├── process.ash      # Process execution
├── time.ash         # Time and date
├── json.ash         # JSON processing
├── regex.ash        # Regular expressions
├── crypto.ash       # Cryptographic hashing
├── mailbox.ash      # Inter-workflow messaging
├── cron.ash         # Scheduled execution
└── log.ash          # Observability/logging
```

---

## std::io - File System Operations

```ash
-- Read operations (effect: read)
capability read_file: read(path: String) returns String
capability read_file_bytes: read(path: String) returns List<Int>
capability file_exists: read(path: String) returns Bool
capability read_dir: read(path: String) returns List<DirEntry>
capability file_metadata: read(path: String) returns FileMetadata

-- Write operations (effect: write)
capability write_file: write(path: String, content: String)
capability write_file_bytes: write(path: String, content: List<Int>)
capability append_file: write(path: String, content: String)
capability create_dir: write(path: String)
capability delete_file: write(path: String)
capability delete_dir: write(path: String)

-- Types
type DirEntry {
    name: String,
    path: String,
    is_file: Bool,
    is_dir: Bool,
    size: Int,
    modified: Time
}

type FileMetadata {
    size: Int,
    created: Time,
    modified: Time,
    accessed: Time,
    is_file: Bool,
    is_dir: Bool,
    permissions: FilePermissions
}

type FilePermissions {
    owner_read: Bool,
    owner_write: Bool,
    owner_execute: Bool,
    group_read: Bool,
    group_write: Bool,
    group_execute: Bool,
    other_read: Bool,
    other_write: Bool,
    other_execute: Bool
}
```

---

## std::net - Network Operations

```ash
-- HTTP operations (effect: external)
capability http_get: external(url: String, headers: List<(String, String)>) returns HttpResponse
capability http_post: external(url: String, body: String, headers: List<(String, String)>) returns HttpResponse
capability http_put: external(url: String, body: String, headers: List<(String, String)>) returns HttpResponse
capability http_delete: external(url: String, headers: List<(String, String)>) returns HttpResponse

-- WebSocket (effect: external)
capability websocket_connect: external(url: String) returns WebSocket
capability websocket_send: external(socket: WebSocket, message: String)
capability websocket_receive: external(socket: WebSocket) returns String
capability websocket_close: external(socket: WebSocket)

-- Types
type HttpResponse {
    status: Int,
    status_text: String,
    headers: List<(String, String)>,
    body: String,
    body_bytes: List<Int>
}

type WebSocket {
    id: String,
    url: String,
    state: WebSocketState
}

type WebSocketState {
    Connecting,
    Open,
    Closing,
    Closed
}
```

---

## std::process - Process Execution

```ash
-- Process operations (effect: external)
capability execute: external(command: String, args: List<String>) returns ProcessOutput
capability execute_shell: external(script: String) returns ProcessOutput
capability spawn: external(command: String, args: List<String>) returns Process

-- Process management (effect: external)
capability process_wait: external(process: Process, timeout: Int) returns ProcessResult
capability process_kill: external(process: Process)
capability process_status: external(process: Process) returns ProcessStatus

-- Types
type ProcessOutput {
    stdout: String,
    stderr: String,
    exit_code: Int,
    success: Bool
}

type Process {
    id: Int,
    command: String,
    args: List<String>
}

type ProcessResult {
    Completed { output: ProcessOutput },
    TimedOut,
    Killed
}

type ProcessStatus {
    Running,
    Exited { code: Int },
    Signaled { signal: Int },
    Stopped,
    Zombie
}
```

---

## std::time - Time and Date

```ash
-- Observation (effect: observe)
capability now: observe() returns DateTime
capability now_utc: observe() returns DateTime
capability elapsed: observe(start: DateTime) returns Duration

-- Analysis (effect: analyze)
capability format_time: analyze(dt: DateTime, format: String) returns String
capability parse_time: analyze(s: String, format: String) returns DateTime
capability add_duration: analyze(dt: DateTime, duration: Duration) returns DateTime
capability sub_duration: analyze(dt: DateTime, duration: Duration) returns DateTime

-- External (effect: external)
capability sleep: external(duration: Duration)
capability sleep_until: external(target: DateTime)

-- Types
type DateTime {
    timestamp: Int,
    year: Int,
    month: Int,
    day: Int,
    hour: Int,
    minute: Int,
    second: Int,
    nanosecond: Int,
    timezone: String
}

type Duration {
    seconds: Int,
    nanoseconds: Int
}
```

---

## std::json - JSON Processing

```ash
-- Analysis (effect: analyze)
capability json_parse: analyze(s: String) returns JsonValue
capability json_stringify: analyze(value: JsonValue) returns String
capability json_stringify_pretty: analyze(value: JsonValue) returns String
capability json_query: analyze(value: JsonValue, path: String) returns JsonValue

-- Types
type JsonValue {
    Null,
    Bool(Bool),
    Number(Float),  -- JSON numbers are f64
    String(String),
    Array(List<JsonValue>),
    Object(Map<String, JsonValue>)
}
```

---

## std::regex - Regular Expressions

```ash
-- Analysis (effect: analyze)
capability regex_match: analyze(pattern: String, text: String) returns List<Match>
capability regex_match_one: analyze(pattern: String, text: String) returns Option<Match>
capability regex_replace: analyze(pattern: String, text: String, replacement: String) returns String
capability regex_replace_all: analyze(pattern: String, text: String, replacement: String) returns String
capability regex_split: analyze(pattern: String, text: String) returns List<String>
capability regex_is_match: analyze(pattern: String, text: String) returns Bool

-- Types
type Match {
    start: Int,
    end: Int,
    matched: String,
    groups: List<String>
}
```

---

## std::crypto - Cryptographic Hashing

```ash
-- Analysis (effect: analyze)
capability sha256: analyze(data: List<Int>) returns List<Int>
capability sha256_string: analyze(s: String) returns List<Int>
capability blake3: analyze(data: List<Int>) returns List<Int>
capability blake3_string: analyze(s: String) returns List<Int>
capability hmac_sha256: analyze(data: List<Int>, key: List<Int>) returns List<Int>

-- Utilities (effect: analyze)
capability hex_encode: analyze(data: List<Int>) returns String
capability hex_decode: analyze(s: String) returns List<Int>
capability base64_encode: analyze(data: List<Int>) returns String
capability base64_decode: analyze(s: String) returns List<Int>
```

---

## std::mailbox - Inter-Workflow Messaging

```ash
-- External (effect: external)
capability send_message: external(target: WorkflowId, message: Message)
capability send_message_timeout: external(target: WorkflowId, message: Message, timeout: Duration) returns Result<(), SendError>

-- Receive (effect: observe)
capability receive_message: observe() returns Message
capability receive_message_timeout: observe(timeout: Duration) returns Option<Message>
capability try_receive_message: observe() returns Option<Message>

-- Types
type WorkflowId {
    id: String,
    namespace: String
}

type Message {
    sender: WorkflowId,
    payload: Value,
    timestamp: DateTime,
    correlation_id: String
}

type SendError {
    Timeout,
    WorkflowNotFound,
    MailboxFull,
    ChannelClosed
}
```

---

## std::cron - Scheduled Execution

```ash
-- External (effect: external)
capability schedule_job: external(name: String, schedule: String, workflow: WorkflowRef) returns JobId
capability schedule_job_once: external(name: String, at: DateTime, workflow: WorkflowRef) returns JobId
capability cancel_job: external(id: JobId)
capability pause_job: external(id: JobId)
capability resume_job: external(id: JobId)

-- Observation (effect: observe)
capability list_jobs: observe() returns List<Job>
capability get_job: observe(id: JobId) returns Job

-- Types
type JobId {
    id: String
}

type Job {
    id: JobId,
    name: String,
    schedule: String,
    workflow: WorkflowRef,
    state: JobState,
    next_run: Option<DateTime>,
    last_run: Option<DateTime>,
    run_count: Int
}

type JobState {
    Active,
    Paused,
    Completed,
    Failed { reason: String }
}
```

---

## std::log - Observability

```ash
-- External (effect: external)
capability log_debug: external(message: String)
capability log_info: external(message: String)
capability log_warn: external(message: String)
capability log_error: external(message: String)

-- With context
capability log_info_with: external(message: String, context: Map<String, JsonValue>)
capability log_error_with: external(message: String, context: Map<String, JsonValue>, error: String)

-- Metrics (effect: external)
capability metric_counter: external(name: String, value: Int, labels: Map<String, String>)
capability metric_gauge: external(name: String, value: Float, labels: Map<String, String>)
capability metric_histogram: external(name: String, value: Float, labels: Map<String, String>)
```

---

## Type Summary

### Primitive Types
- `Int` - Signed 64-bit integer
- `Float` - 64-bit floating point (IEEE 754)
- `String` - UTF-8 encoded string
- `Bool` - Boolean (`true` / `false`)
- `Time` - Timestamp with nanosecond precision

### Container Types
- `List<T>` - Homogeneous list
- `Map<K, V>` - Key-value map
- `Option<T>` - Optional value (`Some(T)` / `None`)
- `Result<T, E>` - Success/failure (`Ok(T)` / `Err(E)`)

### Special Types
- `Value` - Any Ash value (used for dynamic typing)
- `JsonValue` - JSON-compatible value
- `WorkflowId` - Unique workflow identifier
- `JobId` - Unique scheduled job identifier

---

## Effect Classification Reference

| Effect | Description | Examples |
|--------|-------------|----------|
| `observe` | Read-only observation of system state | `now`, `file_exists` |
| `read` | Read data from external source | `read_file`, `http_get` |
| `analyze` | Pure computation, no external effects | `json_parse`, `sha256` |
| `write` | Destructive modification | `write_file`, `delete_file` |
| `external` | External world interaction | `execute`, `sleep`, `send_message` |

---

## Approval Levels

| Capability | Effect | Approval |
|------------|--------|----------|
| `read_file` | read | Never |
| `write_file` | write | UnlessAutoApproved |
| `execute` | external | Always |
| `http_get` | external | UnlessAutoApproved |
| `json_parse` | analyze | Never |
| `sleep` | external | Never |
| `send_message` | external | UnlessAutoApproved |
| `schedule_job` | external | Always |
| `log_info` | external | Never |

---

## Open Questions

1. **Module imports**: How do capabilities get brought into workflow scope?
   - `import std::io` brings all io capabilities?
   - Explicit import: `from std::io import read_file, write_file`?

2. **Type aliases**: Should we support `type FilePath = String` for documentation?

3. **Error types**: Should capabilities declare specific error types or use generic `Error`?

4. **Generic capabilities**: Should stdlib have generic capabilities like `List<T>` operations?
   - Or are those built into the language?

5. **Capability versions**: How do we handle breaking changes in base capabilities?

---

**Next Steps**:
1. Define the import/module system
2. Design the `AshBridge` trait for stdlib types
3. Prototype `read_file` end-to-end
4. Define error type conventions
