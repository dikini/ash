# TASK-047: Example Workflow Library

## Status: 🟢 Complete

## Description

Create a comprehensive library of example Ash workflows demonstrating language features and best practices.

## Specification Reference

- SPEC-002: Surface Language
- SHARO_CORE_LANGUAGE.md - Section 10. Example Workflows

## Requirements

### Example Categories

1. **Basic Examples**
   - Hello world
   - Variable binding
   - Basic expressions
   - Simple observations

2. **Control Flow Examples**
   - Conditionals (if/then/else)
   - Loops (foreach)
   - Parallel execution
   - Sequential composition

3. **Policy Examples**
   - Role-based policies
   - Time-based policies
   - Threshold policies
   - Composite policies

4. **Real-World Examples**
   - Customer support workflow
   - Code review workflow
   - Data processing pipeline
   - Multi-agent research

5. **Advanced Examples**
   - Error handling (try/catch)
   - Retries and timeouts
   - Complex decisions
   - Obligations and checks

### Example Structure

Each example should include:
- Complete workflow source
- Description of what it does
- Expected inputs/outputs
- Explanation of key concepts

```
examples/
├── README.md
├── 01-basics/
│   ├── 01-hello-world.ash
│   ├── 02-variables.ash
│   ├── 03-expressions.ash
│   └── README.md
├── 02-control-flow/
│   ├── 01-conditionals.ash
│   ├── 02-foreach.ash
│   ├── 03-parallel.ash
│   └── README.md
├── 03-policies/
│   ├── 01-role-based.ash
│   ├── 02-time-based.ash
│   ├── 03-threshold.ash
│   └── README.md
├── 04-real-world/
│   ├── customer-support.ash
│   ├── code-review.ash
│   ├── data-pipeline.ash
│   └── README.md
└── 05-advanced/
    ├── error-handling.ash
    ├── retries.ash
    └── README.md
```

### Example Content

**01-hello-world.ash:**
```ash
-- Basic "Hello, World!" example
-- Demonstrates: literals, variable binding, output

workflow hello_world {
  let message = "Hello, World!";
  act print with message: message;
  done
}
```

**02-variables.ash:**
```ash
-- Variable binding and scoping example
-- Demonstrates: let, patterns, variable reuse

workflow variables {
  -- Simple binding
  let x = 42;
  
  -- Pattern binding (tuple destructuring)
  let (a, b) = (1, 2);
  
  -- Record binding
  let {name, age} = {name: "Alice", age: 30};
  
  act print with message: name;
  done
}
```

**customer-support.ash:**
```ash
-- Customer support ticket resolution workflow
-- Demonstrates: observe, orient, decide, act, policies

-- Capability declarations
capability fetch_ticket : observe(ticket_id: String) returns Ticket
capability analyze_sentiment : analyze(text: String) returns Sentiment
capability draft_reply : analyze(ticket: Ticket, context: Documents) returns Draft
capability send_email : act(to: Email, subject: String, body: String)

-- Policy declarations
policy high_confidence:
  when confidence > 0.8 and sentiment != "angry"
  then permit
  else require_approval(role: manager)

policy external_communication:
  when recipient.domain in internal_domains
  then permit
  else require_approval(role: manager)

-- Main workflow
workflow support_ticket_resolution {
  -- Epistemic: Gather information
  observe fetch_ticket with ticket_id: $ticket_id as ticket;
  
  -- Deliberative: Analyze
  orient { analyze_sentiment(text: ticket.description) } as sentiment;
  orient { analyze(ticket, docs) } as analysis;
  
  -- Deliberative: Draft response
  propose draft_reply(ticket, docs) as draft;
  
  -- Evaluative: Policy check
  decide { analysis.confidence } under high_confidence then {
    
    -- Operational: Send (with policy guard)
    act send_email(
      to: ticket.customer_email,
      subject: "Re: " + ticket.subject,
      body: draft.content
    ) where external_communication;
    
  } else {
    -- Escalation path
    act escalate(to: senior_agent, reason: "low_confidence");
  }
  
  done
}
```

**code-review.ash:**
```ash
-- Code review workflow with role separation
-- Demonstrates: roles, obligations, par

-- Role definitions
role drafter {
  authority: [read_code, create_pr, respond_to_comments],
  obligations: [ensure_tests_pass],
  supervises: []
}

role reviewer {
  authority: [read_code, comment, request_changes, approve],
  obligations: [check_tests, check_security, review_logic],
  supervises: drafter
}

-- Capabilities
capability fetch_pr : observe(pr_id: ID) returns PR
capability analyze_diff : analyze(pr: PR) returns Analysis
capability check_coverage : analyze(tests: TestSuite) returns Coverage
capability request_changes : act(pr: PR, comments: List<Comment>)
capability merge_pr : act(pr: PR) where all_checks_pass

-- Workflow
workflow code_review {
  let pr = observe fetch_pr with pr_id: $input.pr_id;
  
  -- Parallel analysis
  par {
    orient analyze_diff(pr) as diff_analysis;
    orient check_coverage(pr.tests) as coverage
  };
  
  -- Obligations for reviewer
  oblige reviewer to check_tests(pr);
  oblige reviewer to check_security(pr);
  
  -- Decision based on analysis
  decide { coverage.percentage > 80 and diff_analysis.no_critical_issues } then {
    
    if diff_analysis.has_minor_issues then {
      act request_changes(pr, comments: diff_analysis.issues);
    } else {
      act merge_pr(pr) where reviewer_approved;
    }
    
  } else {
    act request_changes(
      pr, 
      comments: ["Coverage insufficient", "Critical issues found"]
    );
  }
  
  done
}
```

### Example Testing

```rust
/// Test that all examples parse and type-check
#[test]
fn test_all_examples() {
    let example_dir = Path::new("examples");
    
    for entry in fs::read_dir(example_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.extension().map_or(false, |e| e == "ash") {
            let source = fs::read_to_string(&path).unwrap();
            
            // Parse
            let program = parse(&source)
                .expect(&format!("Failed to parse {}", path.display()));
            
            // Type check
            let errors = type_check(&program);
            assert!(
                errors.is_empty(),
                "Type errors in {}: {:?}",
                path.display(),
                errors
            );
        }
    }
}
```

## TDD Steps

### Step 1: Create Directory Structure

Set up examples/ directory.

### Step 2: Write Basic Examples

Create 01-basics/ examples.

### Step 3: Write Control Flow Examples

Create 02-control-flow/ examples.

### Step 4: Write Policy Examples

Create 03-policies/ examples.

### Step 5: Write Real-World Examples

Create 04-real-world/ examples.

### Step 6: Write Advanced Examples

Create 05-advanced/ examples.

### Step 7: Write README Files

Add documentation for each category.

### Step 8: Add Example Tests

Add test to verify all examples.

## Completion Checklist

- [ ] Directory structure
- [ ] Basic examples (3+)
- [ ] Control flow examples (3+)
- [ ] Policy examples (3+)
- [ ] Real-world examples (3+)
- [ ] Advanced examples (2+)
- [ ] README files
- [ ] Example test
- [ ] All examples parse
- [ ] All examples type-check

## Self-Review Questions

1. **Coverage**: Do examples cover all major features?
2. **Quality**: Are examples well-documented?
3. **Correctness**: Do examples work?

## Estimated Effort

8 hours

## Dependencies

- ash-parser
- ash-typeck

## Blocked By

- TASK-016: Lowering (to verify examples)

## Blocks

- TASK-048: Tutorial
