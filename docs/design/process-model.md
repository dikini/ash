# Ash Process Model: Communicating, Short-Lived, Isolated Processes

## Status

Design note. Explores Ash's process model: a system as a collection of communicating, mostly short-lived, small processes with no shared memory.

## Summary

Ash programs are **systems of communicating processes**. Each process is:
- **Isolated**: No shared memory with other processes
- **Short-lived**: Spawns, computes, terminates, memory released
- **Small**: Minimal memory footprint, near-constant memory usage
- **Effectful**: Communicates via channels (message passing)

Memory management is **region-based at the process level**: all memory owned by a process is released on termination. No GC, no reference counting.

## References

- [DESIGN-NOTE-COMONADIC-COMPUTATION.md](DESIGN-NOTE-COMONADIC-COMPUTATION.md) — comonadic streams as process outputs
- [effectful-stream-sinks.md](effectful-stream-sinks.md) — effectful stream sinks, channel-based communication
- [effect-handling-styles.md](effect-handling-styles.md) — effect handlers for process supervision
- [multi-shot-continuations.md](multi-shot-continuations.md) — multi-shot continuations for pure computations
- Erlang/OTP — actor model, process isolation, message passing
- Rust async/await — cooperative scheduling, ownership-based concurrency
- Capability security (Mark Miller) — isolation as a security primitive

## The Vision

> An Ash program is a collection of communicating, mostly short-lived, small processes.

```text
System
├── Process A (sensor reader)
│   └── Channel A → B (temperature data)
├── Process B (aggregator)
│   └── Channel B → C (alerts)
├── Process C (network sender)
│   └── Channel C → D (acknowledgments)
└── Process D (supervisor)
    └── Monitors A, B, C via effect handlers
```

No shared memory. All communication is by value via channels.

## Memory Model: Region-Based, Per-Process

### No Shared Memory

```ash
-- Process A: computes data, sends to Process B
fn process_a(output: Channel<Float>) -> {cap channel.send} Unit {
    do {
        let data = compute_temperature();  -- owned by Process A
        channel.send(output, data);        -- moves ownership to Process B
        -- data is no longer accessible here
    }
}

-- Process B: receives data, processes it
fn process_b(input: Channel<Float>) -> {cap channel.receive} Unit {
    do {
        let msg = channel.receive(input);    -- msg is now owned by Process B
        let alert = check_threshold(msg);
        -- Process A cannot access msg, no shared references
    }
}
```

### Memory Release on Termination

```text
Process lifecycle:
1. Spawn: allocate memory region for process
2. Run: process owns all its memory, no individual deallocation
3. Terminate: release ALL memory in the region
4. No GC, no reference counting, no leaks
```

```ash
-- Short-lived process: compute and die
fn compute_task(data: Int) -> Int {
    let result = expensive_computation(data);
    result  -- sent back to parent, process terminates, memory freed
}

-- Long-lived process: loop until shutdown
fn server(input: Channel<Request>) -> Unit {
    loop {
        let msg = channel.receive(input);
        match msg {
            Request(data) => {
                let result = handle_request(data);
                channel.send(sender, result);
                -- data and result are dropped after send
            },
            Shutdown => {
                return;  -- process terminates, all memory freed
            }
        }
    }
}
```

## Language Support for Constant Memory

### Ownership and Move Semantics

```ash
-- Values are owned by a process
let data = [1, 2, 3];  -- owned by current process

-- Send moves ownership to another process
channel.send(other_process, data);
-- data is no longer accessible here (compile-time error if used)

-- Receive takes ownership
let received = channel.receive();
-- received is now owned by this process
```

### No Shared References Across Processes

```ash
-- ILLEGAL: shared reference across processes
let shared = ref([1, 2, 3]);
channel.send(other_process, shared);  -- ERROR: cannot share references

-- LEGAL: send by value
let data = [1, 2, 3];
channel.send(other_process, data);  -- OK: moves ownership
```

### Process-Local Types

```ash
-- A type that cannot escape its process
sealed type ProcessLocal<T> = T;

-- Values of this type cannot be sent across channels
fn send(data: ProcessLocal<Int>) -> {cap channel.send} Unit {
    channel.send(other_process, data);  -- ERROR: ProcessLocal cannot be sent
}
```

### Near-Constant Memory Patterns

```ash
-- GOOD: streaming processing, constant memory
fn stream_process(input: Channel<Chunk>, output: Channel<Chunk>) -> Unit {
    loop {
        let chunk = channel.receive(input);
        let processed = transform_chunk(chunk);
        channel.send(output, processed);
        -- chunk and processed are dropped after each iteration
    }
}

-- BAD: accumulating data (compiler warns or errors)
fn process_batch(input: Channel<Int>) -> List<Int> {
    let mut acc = [];
    loop {
        let msg = channel.receive(input);
        acc = Cons(msg, acc);  -- WARNING: unbounded accumulation
    }
    acc
}
```

## Process Model

### Spawning

```ash
-- Spawn a new process
let pid = spawn(fn() => {
    -- new process runs this function
    process_b()
});

-- Send message to process
channel.send(pid, "hello");

-- Receive message
let msg = channel.receive();
```

### Process Effects

```ash
effect Process
  fun spawn(f: () -> {Process} Unit) : Pid
  fun self() : Pid
  fun exit(reason: ExitReason) : Unit

effect Channel<a>
  fun send(pid: Pid, msg: a) : Unit
  fun receive() : a
  fun select(channels: List<Channel<a>>) : (Channel<a>, a)
```

### Effect Rows for Process Isolation

```ash
-- A pure function has no process effects
fn pure_add(x: Int, y: Int) -> Int {
    x + y
}

-- An effectful function can spawn processes
fn spawn_worker() -> {Process} Pid {
    spawn(fn() => {
        do_work()
    })
}

-- A function that uses channels
fn communicate() -> {Process, Channel<String>} Unit {
    let pid = spawn(fn() => {
        let msg = receive();
        println(msg)
    });
    send(pid, "hello")
}
```

## Design Decisions and Answers

### 1. Process Scheduling: Cooperative or Preemptive?

**Answer: Cooperative under the hood, not exposed to the user.**

Rust (the implementation language) uses cooperative scheduling for async/await. Ash follows this model initially. Some effects may trigger an implicit yield, but the user does not control scheduling directly.

Long-term: may consider preemptive scheduling for fairness, but this is a runtime concern, not a language concern.

### 2. Process Size: How Small is "Small"?

**Answer: As small as possible, but not too small to be useful or readable.**

Not a critical question. The language does not enforce a size limit. The programmer decides what constitutes a process. The runtime optimizes for small processes, but the language is agnostic.

### 3. Channel Semantics: Bounded or Unbounded?

**Answer: Adopt initial Rust channel semantics (bounded), but long-term possibly unbounded.**

Rust channels are bounded (backpressure via `send` blocking). This prevents memory exhaustion from slow consumers. Erlang channels are unbounded (risk of memory exhaustion).

Bounded channels align with Ash's constant-memory philosophy.

### 4. Error Handling: Let It Crash, or Effect Rows?

**Answer: Both. In Ash we have `fail`. Let fails fail.**

The `fail` effect aborts the current process. The supervisor (via effect handlers) decides whether to restart, escalate, or terminate. This is Erlang's "let it crash" philosophy, but with static effect tracking.

```ash
-- Process that may fail
fn risky_computation() -> {fail} Int {
    do {
        if random() > 0.5 {
            fail("computation failed")
        } else {
            return 42
        }
    }
}

-- Supervisor: handles failure via effect handler
fn supervisor(action: {fail} Int) -> Int {
    handle action with {
        fail(msg) => {
            log("Process failed: " ++ msg);
            restart_process();  -- restart the process
            supervisor(action)  -- retry
        }
    }
}
```

### 5. Process Monitoring: How Does a Parent Know a Child Crashed?

**Answer: Effect handlers.**

The parent installs a handler that intercepts the child's `fail` effect. The handler is notified when the child fails.

```ash
-- Parent monitors child via handler
fn parent() -> Unit {
    let child = spawn(fn() => {
        risky_computation()
    });

    -- Install handler to monitor child
    monitor(child, {
        fail(msg) => {
            log("Child crashed: " ++ msg);
            restart_child(child)
        }
    });
}
```

This is **effect handler-based supervision**, not Erlang's link/monitor mechanism. The effect row tracks the failure effect.

### 6. Hot Code Reload: Supported?

**Answer: Not initially (start-stop), but no particular obstacles long-term.**

Hot code reload requires:
- Dynamic linking/loading of process code
- Versioning of process interfaces
- Migration of process state between versions

None of these are fundamentally incompatible with Ash's design. They are implementation complexity, not language design issues.

### 7. Distributed Processes: Channels Across Network?

**Answer: Secondary concern, currently explicit.**

Distributed channels are not transparent (unlike Erlang). If a channel spans a network boundary, the programmer must explicitly serialize/deserialize messages and handle network failures.

Long-term: may provide distributed channel abstraction, but this is a library concern, not a language primitive.

### 8. Memory Fragmentation: If Processes Allocate and Free Frequently?

**Answer: Possibly. Long-term: process affinities, regions, and defragmentation.**

If processes spawn and die frequently, memory fragmentation may occur. Mitigation strategies:
- **Process affinities**: Pin processes to system threads/arenas
- **Region allocation**: Each process gets a region; regions are recycled
- **Defragmentation service**: A background process compacts memory per arena

These are runtime concerns, not language concerns. The language guarantees that process termination releases all memory.

## Comparison with Erlang/OTP and Rust

| Aspect | Erlang/OTP | Rust (async) | Ash (Proposed) |
|--------|-----------|--------------|----------------|
| **Process model** | Actor (preemptive) | Task (cooperative) | Actor (cooperative initially) |
| **Memory** | Per-process GC | Ownership + RAII | Region-based (release on termination) |
| **Shared memory** | No | No (Send/Sync traits) | No (move semantics) |
| **Communication** | Message passing | Channels | Channels (effect-based) |
| **Supervision** | Supervisor trees | N/A (manual) | Effect handlers |
| **Fault tolerance** | Let it crash | Panic/abort | `fail` effect + handlers |
| **Hot reload** | Yes | No | Not initially |
| **Distribution** | Transparent | Explicit | Explicit |

## The Comonad-Process Connection

A process's **output** is a **stream** (comonad). A process's **input** is a **channel** (monad/effect).

```ash
-- Process as a comonadic stream producer
fn sensor_process() -> {Yield<Float>} Unit {
    do {
        loop {
            let reading = read_sensor();
            yield(reading);  -- produces a stream element
        }
    }
}

-- Process as a monadic stream consumer
fn alert_process(input: Channel<Float>) -> {cap channel.send} Unit {
    do {
        loop {
            let reading = channel.receive(input);
            if reading > threshold {
                channel.send(alert_channel, Alert(reading));
            }
        }
    }
}

-- Composition: connect producer to consumer
fn system() -> Unit {
    let sensor = spawn(sensor_process);
    let alert = spawn(fn() => alert_process(sensor));
    -- sensor's yield feeds alert's receive
}
```

The process model is the **bridge between comonads (streams) and monads (effects)**.

## Open Questions

1. Should `spawn` be a built-in effect or a library function?
2. Should channels be typed by their message type (e.g., `Channel<Int>`) or by their protocol (e.g., `Channel<Request, Response>`)?
3. How does process supervision compose with nested handlers?
4. Should the compiler warn on unbounded accumulation within a process?
5. How do we express process pools (e.g., worker pools) in Ash?
6. What is the syntax for process-local state (e.g., `let process var x = 0`)?
7. How does the process model interact with the capability system (e.g., can a process hold a capability)?

## Changelog

- 2026-06-20: Created design note exploring Ash's process model: communicating, short-lived, isolated processes with region-based memory management.
