# Ash Tutorial

A step-by-step guide to learning the Ash workflow language.

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Your First Workflow](#your-first-workflow)
4. [Variables and Patterns](#variables-and-patterns)
5. [Expressions](#expressions)
6. [Control Flow](#control-flow)
7. [The OODA Pattern](#the-ooda-pattern)
8. [Policies and Roles](#policies-and-roles)
9. [Real-World Example](#real-world-example)
10. [Next Steps](#next-steps)

## Introduction

Ash is a workflow language designed for governed AI systems. It provides:

- **Strong typing** with type inference
- **Effect tracking** for capability safety
- **Policy enforcement** for governance
- **Provenance tracking** for auditability
- **The OODA pattern** (Observe-Orient-Decide-Act) for structuring workflows

### Why Ash?

Traditional workflow languages often lack:

- Built-in governance mechanisms
- Fine-grained capability control
- Audit trails by default
- Policy-based decision making

Ash addresses these gaps by making governance a first-class concern.

## Installation

### Prerequisites

- Rust 1.94.0 or later
- Cargo (included with Rust)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/dikini/ash
cd ash

# Build the project
cargo build --release

# Install the CLI (optional)
cargo install --path crates/ash-cli
```

### Verify Installation

```bash
# Check version
ash --version

# View help
ash --help
```

## Your First Workflow

Create a file called `hello.ash`:

```ash
workflow main {
    ret "Hello, World!"
}
```

### Running the Workflow

```bash
# Type check
ash check hello.ash

# Run
ash run hello.ash
```

### Understanding the Structure

```ash
workflow main {       // Workflow declaration with name
    ret "Hello!"      // Return statement
}
```

- `workflow` - Declares a workflow
- `main` - The workflow name (entry point)
- `ret` - Returns a value

## Variables and Patterns

### Simple Binding

```ash
workflow main {
    let name = "Alice"
    let age = 30
    ret { name: name, age: age }
}
```

### Pattern Matching

```ash
workflow main {
    // Tuple destructuring
    let (x, y) = (10, 20)
    
    // Record destructuring
    let person = { name: "Bob", age: 25 }
    let { name: n, age: a } = person
    
    // List patterns
    let items = [1, 2, 3, 4, 5]
    let [first, second, ..rest] = items
    
    // Wildcard (ignores value)
    let _ = "ignored"
    
    ret { x: x, y: y, name: n, first: first }
}
```

### Pattern Types

| Pattern | Description | Example |
|---------|-------------|---------|
| Variable | Binds to a name | `let x = 5` |
| Tuple | Destructures tuples | `let (a, b) = (1, 2)` |
| Record | Destructures records | `let {x, y} = point` |
| List | Destructures lists | `let [h, ..t] = list` |
| Wildcard | Ignores value | `let _ = unused` |

## Expressions

### Literals

```ash
let integer = 42
let floating = 3.14
let boolean = true
let string = "hello"
let null_value = null
```

### Arithmetic

```ash
let sum = 10 + 20
let diff = 50 - 10
let product = 5 * 4
let quotient = 20 / 4
let modulo = 17 % 5
```

### Comparison

```ash
let eq = (10 == 10)      // true
let ne = (10 != 5)       // true
let lt = (5 < 10)        // true
let gt = (10 > 5)        // true
let le = (5 <= 5)        // true
let ge = (10 >= 5)       // true
```

### Logical

```ash
let and = true && false   // false
let or = true || false    // true
let not = !true           // false
```

### Conditional Expressions

```ash
let score = 85
let grade = if score >= 90 { "A" }
            else if score >= 80 { "B" }
            else if score >= 70 { "C" }
            else { "F" }
```

## Control Flow

### Sequential Execution

Workflows execute statements in order:

```ash
workflow main {
    let a = 1
    let b = 2
    let c = a + b
    ret c  // 3
}
```

### Conditional Workflow

```ash
workflow main {
    let temperature = 75
    
    if temperature > 80 {
        ret "hot"
    } else if temperature > 60 {
        ret "comfortable"
    } else {
        ret "cold"
    }
}
```

### Loops

```ash
workflow main {
    let numbers = [1, 2, 3, 4, 5]
    let sum = 0
    
    for n in numbers {
        let sum = sum + n
    }
    
    ret sum  // 15
}
```

### Parallel Execution

```ash
workflow main {
    par {
        // Branch 1
        let result1 = compute_a()
        
        // Branch 2
        let result2 = compute_b()
        
        // Branch 3
        let result3 = compute_c()
    }
    
    ret { a: result1, b: result2, c: result3 }
}
```

## The OODA Pattern

OODA (Observe-Orient-Decide-Act) is the core pattern for Ash workflows.

### Observe

Read data from the outside world:

```ash
capability sensor {
    effect: observe,
    params: [id: String],
    returns: Reading
}

workflow main {
    observe sensor("temp_01") as reading
    ret reading
}
```

### Orient

Analyze and transform data:

```ash
orient {
    let status = if reading.value > 80 {
        "critical"
    } else {
        "normal"
    }
} as analysis
```

### Decide

Apply policies to make decisions:

```ash
policy high_priority {
    condition: analysis.status == "critical",
    decision: escalate
}

decide {
    if analysis.status == "critical" {
        action "send_alert"
    } else {
        action "log_reading"
    }
}
```

### Act

Execute actions with provenance:

```ash
act send_notification {
    recipient: "ops@example.com",
    message: "Critical temperature: " + reading.value,
    timestamp: now()
} with guard always
```

### Complete OODA Example

```ash
capability read_sensor {
    effect: observe,
    params: [sensor_id: String],
    returns: SensorData
}

policy critical_threshold {
    condition: data.temperature > 80,
    decision: escalate
}

workflow monitor_temperature {
    // OBSERVE
    observe read_sensor("temp_01") as data
    
    // ORIENT
    orient {
        let status = if data.temperature > 80 { "critical" }
                    else if data.temperature > 60 { "warning" }
                    else { "normal" }
        let trend = calculate_trend(data.history)
        { status: status, trend: trend }
    } as analysis
    
    // DECIDE
    decide {
        if analysis.status == "critical" {
            action "immediate_alert"
        } else if analysis.status == "warning" {
            action "schedule_check"
        } else {
            action "log_only"
        }
    }
    
    // ACT
    if action == "immediate_alert" {
        act send_alert {
            level: "critical",
            sensor: "temp_01",
            value: data.temperature
        }
    }
    
    ret { data: data, analysis: analysis, action: action }
}
```

## Policies and Roles

The examples in this section are high-level governance sketches. Treat them as reference-oriented
examples; the canonical current surface syntax and role contract live in `docs/spec/`.

### Defining Roles

```ash
role admin {
    authority: [read, write, delete, manage_users]
}

role manager {
    authority: [read, write, approve]
}

role user {
    authority: [read, write]
}
```

### Defining Capabilities

```ash
capability delete_record {
    effect: delete,
    params: [record_id: String],
    requires: role(admin)
}

capability edit_record {
    effect: write,
    params: [record_id: String, changes: Record],
    requires: any_role([admin, manager, user])
}
```

### Defining Policies

```ash
policy can_delete {
    condition: user.role == admin && !record.protected,
    decision: permit
}

policy needs_approval {
    condition: record.sensitivity == "high",
    decision: require_approval(role: manager)
}

policy auto_approve {
    condition: record.owner == user.id && record.sensitivity == "low",
    decision: permit
}
```

### Using Policies in Workflows

```ash
workflow process_request {
    observe fetch_record(record_id) as record
    
    decide {
        if record.sensitivity == "high" {
            action "request_approval"
        } else if current_user().role == admin {
            action "direct_delete"
        } else {
            action "deny"
        }
    }
    
    if action == "deny" {
        ret { error: "Access denied" }
    }
    
    // Proceed with action...
}
```

## Real-World Example

Let's build a complete customer support workflow:

This end-to-end scenario is also reference-oriented rather than a canonical surface-syntax
conformance sample.

```ash
// Capabilities
capability fetch_ticket {
    effect: read,
    params: [ticket_id: String],
    returns: Ticket
}

capability send_email {
    effect: act,
    params: [to: String, subject: String, body: String]
}

// Roles
role supervisor {
    authority: [view_all, assign, resolve],
    obligations: []
}

role agent {
    authority: [view_assigned, respond, resolve],
    obligations: [respond_within_sla]
}

// Policies
policy urgent_ticket {
    condition: ticket.priority == "urgent",
    decision: escalate
}

workflow support_ticket {
    // Observe: Fetch ticket data
    observe fetch_ticket(ticket_id) as ticket
    
    // Orient: Analyze priority and sentiment
    orient {
        let priority = if ticket.priority == "urgent" { 100 }
                      else if ticket.priority == "high" { 75 }
                      else { 50 }
        let sentiment = analyze_sentiment(ticket.description)
        { priority: priority, sentiment: sentiment }
    } as analysis
    
    // Decide: Route based on analysis
    decide {
        if analysis.priority >= 100 {
            action "escalate_immediately"
        } else if analysis.sentiment < -0.5 {
            action "priority_queue"
        } else {
            action "standard_queue"
        }
    }
    
    // Act: Assign and notify
    if action == "escalate_immediately" {
        act send_email {
            to: find_supervisor().email,
            subject: "URGENT: Ticket " + ticket_id,
            body: "Please review immediately."
        }
    }
    
    oblige agent respond_within_sla {
        ticket: ticket_id,
        deadline: sla_deadline(ticket)
    }
    
    ret { ticket: ticket, assigned: true }
}
```

## Next Steps

### Explore Examples

Look at the complete examples in the `examples/` directory:

For the current canonical syntax contract, prefer `docs/spec/` plus the smaller introductory
examples earlier in this tutorial. The larger policy and scenario sketches here and in
`examples/` are intentionally reference-oriented design examples rather than surface-syntax
conformance samples.

```bash
ls examples/
```

### Read the API Documentation

See [API.md](API.md) for detailed API reference.

### Learn Advanced Topics

- **Effect System**: Understanding capability safety
- **Type System**: Advanced types and constraints
- **Provenance**: Audit trails and compliance
- **Testing**: Writing tests for workflows

### Join the Community

- GitHub: <https://github.com/dikini/ash>
- Issues: Report bugs and request features
- Discussions: Ask questions and share ideas

---

**Congratulations!** You've learned the basics of Ash. Now go build something amazing!
