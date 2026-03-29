# Threading Model Design

**Date:** 2026-03-28
**Status:** Initial Design (First Iteration)
**Related:**
- Erlang/BEAM scheduling principles
- Tokio runtime patterns
- Spec: Workflow spawning and supervision

---

## Overview

Ash adopts a lightweight process model inspired by Erlang/BEAM, implemented using Rust's Tokio async runtime. Each spawned workflow is a logical process that runs concurrently with other workflows, scheduled by a managed thread pool.

**Key goals:**
1. Fair scheduling - no single workflow starves others
2. Global progress - one slow workflow shouldn't block the system
3. Non-blocking IO - IO operations don't block workflow execution
4. Observability - monitor execution at yield points
5. Resource efficiency - use all available cores without oversubscription

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Tokio Multi-threaded Runtime          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Work-Stealing Scheduler (N worker threads)       │   │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐         │   │
│  │  │ T1   │ │ T2   │ │ T3   │ │ T4   │  ...  │   │
│  │  └──────┘ └──────┘ └──────┘ └──────┘         │   │
│  │     ↓        ↓        ↓        ↓                 │   │
│  │  [Global Task Queue] ← Work stealing             │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────────┐
│              Workflow Task Scheduler                      │
│  ┌──────────────┐  ┌──────────────┐                │
│  │ Workflow 1   │  │ Workflow 2   │   ...          │
│  │ (Tokio task) │  │ (Tokio task) │                │
│  └──────────────┘  └──────────────┘                │
│         ↓                 ↓                            │
│    Mailbox          Mailbox                         │
│  (mpsc channel)  (mpsc channel)                  │
└─────────────────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────────────────┐
│               IO/Blocking Pool                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐            │
│  │ IO Thread │ │ IO Thread │ │ IO Thread │   ...       │
│  └──────────┘ └──────────┘ └──────────┘            │
│  Handles: file reads/writes, process exec, blocking     │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Concepts

### 1. Workflows as Logical Processes

Each spawned workflow is a **logical process**, not an OS thread:

- **Lightweight**: Just a Tokio task (stack allocation, no OS thread)
- **Concurrent**: Multiple workflows run in parallel across available cores
- **Independent**: Each has its own mailbox and control link
- **Managed**: Scheduler ensures fair execution

**Comparison:**

| Aspect               | OS Thread | Ash Workflow (Tokio Task) |
|----------------------|------------|---------------------------|
| Cost to spawn        | High       | Very low                  |
| Stack size           | 8MB default| Small (configurable)      |
| Preemptive switch   | OS scheduler| Tokio scheduler (manual) |
| Count               | Hundreds    | Thousands/Millions         |

### 2. Tokio Runtime Configuration

**Multi-threaded runtime** (primary choice):

```rust
use tokio::runtime::Builder;

fn create_ash_runtime(config: &AshRuntimeConfig) -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(
            config.worker_threads.unwrap_or_else(num_cpus::get)
        )
        .thread_name("ash-worker")
        .thread_stack_size(2 * 1024 * 1024)  // 2MB per thread
        .enable_all()  // IO + time features
        .on_thread_start(|| {
            // Initialize thread-local storage
            // Setup metrics hooks
        })
        .build()
        .expect("Failed to create Ash runtime")
}
```

**Configuration defaults:**

| Parameter             | Default                  | Rationale                          |
|----------------------|--------------------------|-----------------------------------|
| Worker threads        | `num_cpus::get()`       | 1:1 with cores (no oversubscription) |
| Thread stack size    | 2MB                      | Sufficient for Ash code, low overhead |
| IO pool threads      | `num_cpus::get() / 2`   | Half cores for blocking operations     |
| Reduction quota      | 1000                      | Tunable, based on experiments      |
| Mailbox capacity    | 1000 messages              | Bounded, drop on overflow        |

### 3. Reduction Counting for Fairness

Inspired by Erlang's preemptive scheduling, Ash uses **reduction counting** to ensure no single workflow monopolizes a thread.

**How it works:**

1. Each workflow has a `reduction_count` counter
2. Every Ash instruction/operation increments the counter
3. When `reduction_count >= REDUCTION_QUOTA`, the workflow yields
4. Tokio scheduler picks the next runnable workflow

**Implementation:**

```rust
pub struct WorkflowTask {
    id: WorkflowId,
    mailbox: Receiver<Message>,
    control_link: Option<ControlLink>,
    reduction_count: u32,
}

const REDUCTION_QUOTA: u32 = 1000;  // Configurable

impl WorkflowTask {
    pub async fn run(mut self) {
        loop {
            // Process messages from mailbox
            while let Ok(msg) = self.mailbox.try_recv() {
                self.handle_message(msg).await;
                self.reduction_count += 1;
            }

            // Check for explicit Ash yield (await/yield keyword)
            // This also counts as reduction

            // Yield after N reductions
            if self.reduction_count >= REDUCTION_QUOTA {
                self.on_yield();  // Monitoring hook
                tokio::task::yield_now().await;
                self.reduction_count = 0;
            }

            // Check control link for shutdown
            if let Some(link) = &self.control_link {
                if link.check_shutdown().await {
                    break;
                }
            }
        }
    }
}
```

**What counts as a reduction:**

| Operation             | Reductions |
|-----------------------|------------|
| Ash function call     | 1          |
| Message send/receive  | 1          |
| Variable assignment   | 0 or 1*   |
| Loop iteration       | 1*         |

*Implementation detail: May group cheap operations

**Yield points:**

- **Explicit**: `yield` keyword in Ash code
- **Implicit**: After `REDUCTION_QUOTA` operations
- **Blocking**: Any `await` point in capability implementation

**Trade-offs:**

| Quota value | Pros                          | Cons                          |
|-------------|--------------------------------|--------------------------------|
| Low (100)   | Very fair, good responsiveness   | High overhead, less throughput    |
| High (5000)  | Low overhead, good throughput    | Risk of starvation, sluggish    |
| Default (1000) | Balanced                    | May need tuning for workloads   |

**Future work:** Adaptive quota based on workflow profiling, system load, or machine learning models.

---

## Communication

### 1. Inter-workflow Communication (Mailboxes)

Each workflow has a **mailbox** for receiving messages from other workflows.

**Implementation:**

```rust
use tokio::sync::mpsc;

pub fn workflow_mailbox(capacity: usize) -> (Sender<Message>, Receiver<Message>) {
    mpsc::channel(capacity)  // Bounded channel
}
```

**Backpressure policy (first iteration):**

- **Bounded capacity**: 1000 messages (configurable)
- **Drop on full**: Oldest messages dropped first
- **Sender notification**: Optional future - return error on full

**Rationale:**
- Keeps system responsive even under heavy load
- Matches Erlang's bounded mailbox approach
- Simple implementation for first iteration
- Prevents memory exhaustion from runaway senders

**Future options:**
- Return "mailbox full" error to sender (backpressure)
- Block sender until space available (risk of deadlock)
- Priority queues for high/low importance messages

### 2. Control Link (Supervisor Communication)

The control link connects workflows to their supervisor, following the supervision tree specification.

**Implementation:**

```rust
use tokio::sync::watch;

pub struct ControlLink {
    shutdown_rx: watch::Receiver<bool>,
}

impl ControlLink {
    pub async fn check_shutdown(&self) -> bool {
        *self.shutdown_rx.borrow()
    }
}
```

**Supervisor side:**

```rust
pub struct Supervisor {
    children: HashMap<WorkflowId, watch::Sender<bool>>,
}

impl Supervisor {
    pub fn spawn_child(&mut self, workflow_id: WorkflowId) {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        // Spawn child with control link
        spawn_workflow(workflow_id, shutdown_rx);

        // Store for later shutdown
        self.children.insert(workflow_id, shutdown_tx);
    }

    pub async fn shutdown_child(&mut self, workflow_id: WorkflowId) {
        if let Some(tx) = self.children.remove(&workflow_id) {
            let _ = tx.send(true);  // Signal shutdown
        }
    }
}
```

---

## IO Handling

IO operations must **never block workflow threads**. Ash provides two strategies:

### 1. Async IO (Preferred for network/sockets)

Use Tokio's built-in async operations:

```rust
use reqwest::Client;
use tokio::net::TcpStream;

// HTTP requests (async)
pub async fn http_get_async(url: &str) -> Result<String, Error> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    Ok(response.text().await?)
}

// TCP connections (async)
pub async fn tcp_connect_async(addr: &str) -> Result<TcpStream, Error> {
    TcpStream::connect(addr).await.map_err(Error::from)
}
```

**Never blocks:** Tokio uses epoll/kqueue/io_uring internally.

### 2. Blocking IO Pool (Files, processes, syscalls)

For inherently blocking operations, offload to a dedicated thread pool:

```rust
use tokio::task;

pub async fn perform_blocking_io<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    task::spawn_blocking(f).await.unwrap()
}
```

**Usage examples:**

```rust
// File operations
pub async fn read_file_async(path: &str) -> Result<String, Error> {
    perform_blocking_io(|| {
        std::fs::read_to_string(path)
    }).await
        .map_err(Error::from)
}

// Process execution
pub async fn execute_async(cmd: &str) -> Result<ProcessOutput, Error> {
    perform_blocking_io(|| {
        std::process::Command::new(cmd)
            .output()
            .map_err(Error::from)
    }).await
}
```

**Alternative: Async file I/O**

```rust
use tokio::fs;

pub async fn read_file_async(path: &str) -> Result<String, Error> {
    fs::read_to_string(path).await.map_err(Error::from)
}
```

`tokio::fs` internally uses the blocking pool, providing async API with blocking semantics.

### 3. IO Pool Configuration

```rust
pub struct AshRuntimeConfig {
    pub io_threads: Option<usize>,  // None = num_cpus::get() / 2
}

fn create_io_pool(config: &AshRuntimeConfig) -> Runtime {
    // Separate runtime for blocking operations
    Builder::new_multi_thread()
        .worker_threads(config.io_threads.unwrap_or_else(|| num_cpus::get() / 2))
        .thread_name("ash-io")
        .build()
        .expect("Failed to create IO runtime")
}
```

**Why separate IO pool?**
- Isolation: Blocking IO doesn't affect workflow scheduling
- Tunability: Different thread count for IO vs compute
- Observability: Separate metrics for IO operations

---

## Monitoring Hooks

Monitoring is integrated at yield points to track execution behavior.

### Hook Interface

```rust
pub struct MonitoringHooks {
    metrics: MetricsCollector,
}

impl MonitoringHooks {
    pub fn on_yield(&self, workflow_id: WorkflowId, reductions: u32) {
        self.metrics.record_reduction_usage(workflow_id, reductions);
        self.metrics.record_yield_time(Instant::now());
    }

    pub fn on_spawn(&self, workflow_id: WorkflowId, thread_id: usize) {
        self.metrics.record_spawn(workflow_id, thread_id);
    }

    pub fn on_message(&self, workflow_id: WorkflowId, size: usize) {
        self.metrics.record_message_size(workflow_id, size);
    }

    pub fn on_error(&self, workflow_id: WorkflowId, error: &Error) {
        self.metrics.record_error(workflow_id, error);
    }
}
```

### Integration Points

**Spawn:**
```rust
fn spawn_workflow(task: WorkflowTask, hooks: &MonitoringHooks) {
    let thread_id = current_thread_id();
    hooks.on_spawn(task.id, thread_id);
    tokio::spawn(task.run(hooks));
}
```

**Yield:**
```rust
impl WorkflowTask {
    async fn run(mut self, hooks: &MonitoringHooks) {
        loop {
            // ... work ...

            if self.reduction_count >= REDUCTION_QUOTA {
                hooks.on_yield(self.id, self.reduction_count);
                tokio::task::yield_now().await;
                self.reduction_count = 0;
            }
        }
    }
}
```

**Observability tools:**
- tokio-console: Real-time task inspection
- Custom metrics: Prometheus-compatible
- Tracing: Distributed tracing integration

---

## Configuration

### Complete Configuration Structure

```rust
pub struct AshRuntimeConfig {
    // Thread pool configuration
    pub worker_threads: Option<usize>,
    pub thread_stack_size: usize,

    // Reduction counting
    pub reduction_quota: usize,

    // IO pool
    pub io_threads: Option<usize>,

    // Mailbox
    pub mailbox_capacity: usize,
    pub mailbox_overflow_policy: MailboxOverflowPolicy,

    // Monitoring
    pub enable_monitoring: bool,
    pub monitoring_interval: Duration,
}

#[derive(Clone, Copy)]
pub enum MailboxOverflowPolicy {
    DropOldest,
    DropNewest,
    // Future: BlockSender, ReturnError
}

impl Default for AshRuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: None,  // Use num_cpus::get()
            thread_stack_size: 2 * 1024 * 1024,
            reduction_quota: 1000,
            io_threads: None,  // Use num_cpus::get() / 2
            mailbox_capacity: 1000,
            mailbox_overflow_policy: MailboxOverflowPolicy::DropOldest,
            enable_monitoring: true,
            monitoring_interval: Duration::from_secs(1),
        }
    }
}
```

### Runtime Creation

```rust
pub struct AshRuntime {
    config: AshRuntimeConfig,
    runtime: Runtime,
    io_runtime: Runtime,
    monitoring: MonitoringHooks,
}

impl AshRuntime {
    pub fn new(config: AshRuntimeConfig) -> Result<Self, Error> {
        let runtime = create_ash_runtime(&config);
        let io_runtime = create_io_pool(&config);
        let monitoring = MonitoringHooks::new(&config);

        Ok(Self {
            config,
            runtime,
            io_runtime,
            monitoring,
        })
    }

    pub fn spawn_workflow(&self, workflow: Workflow) -> Result<WorkflowId, Error> {
        // Create mailbox
        let (tx, rx) = workflow_mailbox(self.config.mailbox_capacity);

        // Create workflow task
        let task = WorkflowTask::new(workflow, rx);

        // Spawn with monitoring
        self.runtime.block_on(async move {
            self.monitoring.on_spawn(task.id, current_thread_id());
            tokio::spawn(task.run(&self.monitoring));
        });

        Ok(task.id)
    }
}
```

---

## First Iteration Scope

### Implemented Features

- [x] Tokio multi-threaded runtime with configurable worker threads
- [x] Workflows as Tokio tasks (logical processes)
- [x] Manual reduction counting with configurable quota
- [x] Bounded mailboxes with drop-on-overflow policy
- [x] Control links for supervision
- [x] Non-blocking IO via `tokio::fs` and `spawn_blocking`
- [x] Monitoring hooks at yield points
- [x] IO thread pool (blocking operations)

### Explicitly Out of Scope

- [ ] Process stickiness (deferred to future iterations)
- [ ] Spawn limiting (unbounded for now, add if needed)
- [ ] Adaptive reduction quota (needs profiling data)
- [ ] Priority-based scheduling (all workflows equal priority)
- [ ] Work stealing algorithm customization (use Tokio default)
- [ ] Advanced mailbox policies (only drop for now)

### Deferred to Future Work

#### 1. Stickiness

**Problem:** Some workflows benefit from staying on the same thread.

**Use cases:**
- Long-running workflows with thread-local caches
- Workflows with expensive initialization
- NUMA-aware scheduling (future optimization)

**Approach (future):**
- Track "last run thread" per workflow
- Prefer same thread on reschedule
- Still allow work stealing under load

**Complexity:** Moderate. Requires custom scheduler or Tokio task affinity integration.

#### 2. Spawn Limiting

**Problem:** Too many workflows can exhaust system resources.

**Use cases:**
- Rate-limited external API calls
- Memory-constrained environments
- Denial-of-service protection

**Approach (future):**
```rust
if active_workflows >= MAX_WORKFLOWS {
    return Error::TooManyWorkflows;
}
spawn(task);
```

**Complexity:** Low. Simple counter check before spawn.

#### 3. Adaptive Reduction Quota

**Problem:** Fixed quota doesn't adapt to workload characteristics.

**Approach (future):**
- Profile workflow reduction rates
- Increase quota for CPU-intensive workflows
- Decrease quota for I/O-bound workflows
- Machine learning model for optimal quota per workload type

**Complexity:** High. Requires profiling infrastructure and ML model.

---

## Lessons from Erlang/BEAM

### 1. Preemptive on Top of Cooperative

Erlang's key insight: Cooperative at C level, preemptive at process level.

- **Cooperative:** VM can only suspend at certain points (function calls, receives)
- **Preemptive:** After N reductions, process is forcibly suspended

**Ash adoption:** Same pattern via reduction counting + Tokio yield points.

### 2. M:N Scheduling

Erlang: M schedulers (threads) manage N processes (N >> M).

- One scheduler per core
- Processes are lightweight memory structures
- Work stealing for load balancing

**Ash adoption:** Tokio's work-stealing scheduler provides M:N automatically.

### 3. Reduction Counting

Erlang's `CONTEXT_REDS` defines how many "reductions" a process gets before yielding.

- Default: 4000 reductions (OTP 20+)
- Each function call = 1 reduction
- BIFs must yield voluntarily

**Ash adoption:** Similar concept, implemented manually with configurable quota.

### 4. No Blocking in Worker Threads

Erlang drivers and NIFs must never block schedulers.

- Use async driver API for IO
- Dirty schedulers for CPU-heavy operations
- Dirty IO schedulers for blocking syscalls

**Ash adoption:** `tokio::fs` and `spawn_blocking` for all blocking operations.

---

## Comparison with Other Runtimes

| Feature                | Ash (Tokio)    | Erlang/BEAM     | OCaml Lwt     | Haskell GHC     |
|------------------------|-----------------|-------------------|----------------|-----------------|
| Scheduling model       | Work-stealing    | M:N preemptive    | Cooperative     | Work-stealing   |
| Task granularity       | Tokio task      | BEAM process      | Lwt thread     | GHC thread      |
| Preemption           | Manual (yield)   | Reduction count    | Manual         | GHC runtime     |
| IO model             | Async (epoll)   | Async (drivers)    | Async          | Async (MIO)    |
| Blocking IO handling  | spawn_blocking   | Dirty schedulers   | Lwt_preemptive | FFI            |
| Fairness guarantee    | Reduction quota  | Reduction count    | N/A            | GHC runtime     |
| Stickiness           | Future          | None              | N/A            | N/A            |
| Monitoring           | tokio-console   | Observer, recon    | Custom         | GHC events      |

---

## Open Questions

1. **Default reduction quota:** 1000 is a reasonable starting point, but experiments are needed to validate. Consider benchmarking typical workflows.

2. **Mailbox capacity:** 1000 messages is arbitrary. Should be based on:
   - Average message size
   - Expected processing rate
   - Memory constraints

3. **IO pool sizing:** `num_cpus / 2` is heuristic. May need tuning based on:
   - Ratio of IO vs compute work
   - Blocking operation latency

4. **Yield overhead:** Each `tokio::task::yield_now()` has a cost. Measure impact and consider batching yields.

5. **Tooling integration:** How to expose monitoring data to users?
   - tokio-console integration
   - Prometheus metrics
   - Custom API

---

## References

- [The BEAM Book - Scheduling](https://blog.stenmans.org/theBeamBook/)
- [Tokio Runtime Documentation](https://tokio.rs/tokio/topics/runtime)
- [Erlang Efficiency Guide](http://erlang.org/doc/efficiency_guide.html)
- [Lwt Cooperative Threading](https://github.com/ocsigen/lwt)
- [Haskell GHC RTS](https://gitlab.haskell.org/ghc/ghc/-/wikis/commentary/rts/scheduler)

---

## Change Log

- 2026-03-28: Initial design document created
