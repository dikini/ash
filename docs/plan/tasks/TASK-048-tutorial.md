# TASK-048: User Tutorial

## Status: 🟢 Complete

## Description

Create a comprehensive user tutorial that guides new users through learning the Ash workflow language.

## Specification Reference

- SPEC-002: Surface Language
- AGENTS.md - Documentation requirements

## Requirements

### Tutorial Structure

```
docs/tutorial/
├── README.md
├── 00-introduction.md
├── 01-getting-started.md
├── 02-basic-syntax.md
├── 03-effects.md
├── 04-policies.md
├── 05-roles.md
├── 06-control-flow.md
├── 07-error-handling.md
├── 08-testing.md
└── 09-best-practices.md
```

### Tutorial Content

**00-introduction.md:**
- What is Ash?
- Why use Ash?
- Core concepts (effects, policies, provenance)
- Installation

**01-getting-started.md:**
- First workflow
- Running with `ash run`
- Checking with `ash check`
- Viewing traces with `ash trace`

**02-basic-syntax.md:**
- Workflows and definitions
- Variables and patterns
- Expressions
- Capabilities and actions

**03-effects.md:**
- The four effects (epistemic, deliberative, evaluative, operational)
- Effect inference
- Why effects matter
- Common effect patterns

**04-policies.md:**
- What are policies?
- Policy syntax
- Decision types (permit, deny, require_approval, escalate)
- Writing effective policies

**05-roles.md:**
- Role definition
- Authority and obligations
- Role hierarchy
- Separation of duties

**06-control-flow.md:**
- Sequential composition
- Conditionals
- Parallel execution
- Loops

**07-error-handling.md:**
- Try/catch patterns
- Retries
- Timeouts
- Fallback workflows

**08-testing.md:**
- Testing workflows
- Property testing
- Integration testing

**09-best-practices.md:**
- Naming conventions
- Documentation
- Error messages
- Performance tips

### Tutorial Style

- Hands-on: Each section has exercises
- Progressive: Builds on previous sections
- Practical: Real-world examples
- Clear: Minimal jargon, explain terms

### Sample Content

**From 02-basic-syntax.md:**

```markdown
# Basic Syntax

## Your First Workflow

Let's create a simple workflow that observes some data and prints it:

```ash
workflow hello {
  -- Observe data from a capability
  observe read_file with path: "/etc/hostname" as hostname;
  
  -- Print the result
  act print with message: hostname;
  
  done
}
```

Save this as `hello.ash` and run it:

```bash
$ ash run hello.ash
my-computer
```

## Breaking It Down

### The `workflow` Keyword

Every Ash file defines at least one workflow using the `workflow` keyword:

```ash
workflow name {
  -- workflow body
}
```

### Observing Data

The `observe` statement reads data without side effects:

```ash
observe capability_name with argument: value as binding;
```

This:
1. Calls the capability `capability_name`
2. Passes `argument: value`
3. Binds the result to `binding`

### Acting

The `act` statement performs side effects:

```ash
act capability_name with argument: value;
```

### The `done` Keyword

Every workflow must end with `done`.

## Exercises

1. Create a workflow that observes two files and prints both.
2. Create a workflow that uses the `env_var` capability to read `$HOME`.
3. Try removing `done` - what error do you get?
```

## TDD Steps

### Step 1: Create Tutorial Structure

Set up docs/tutorial/ directory.

### Step 2: Write Introduction

Create 00-introduction.md.

### Step 3: Write Getting Started

Create 01-getting-started.md.

### Step 4: Write Basic Syntax

Create 02-basic-syntax.md.

### Step 5: Write Remaining Sections

Create 03-09.

### Step 6: Add Exercises

Add hands-on exercises to each section.

### Step 7: Review and Edit

Review for clarity and completeness.

## Completion Checklist

- [ ] Tutorial structure
- [ ] Introduction section
- [ ] Getting started section
- [ ] Basic syntax section
- [ ] Effects section
- [ ] Policies section
- [ ] Roles section
- [ ] Control flow section
- [ ] Error handling section
- [ ] Testing section
- [ ] Best practices section
- [ ] Exercises in each section
- [ ] Review and edit

## Self-Review Questions

1. **Completeness**: Does the tutorial cover everything a beginner needs?
2. **Clarity**: Is the language clear and jargon-free?
3. **Engagement**: Are the exercises interesting?

## Estimated Effort

8 hours

## Dependencies

- TASK-047: Examples (references examples)

## Blocked By

- TASK-047: Examples

## Blocks

- None
