# TASK-013: Workflow Parser

## Status: 🟢 Complete

## Description

Implement parsers for workflow definitions and statements using the winnow combinator infrastructure.

## Specification Reference

- SPEC-002: Surface Language - Section 3.5 Workflow Definition

## Requirements

### Workflow Definition Parser

```rust
/// Parse a complete workflow definition
/// workflow <name> { <body> }
pub fn workflow_def<'a>(input: &mut ParseInput<'a>) -> PResult<WorkflowDef, ParseError> {
    let start = *input;
    
    keyword("workflow").parse_next(input)?;
    let name = token(identifier).parse_next(input)?;
    let body = delimited(
        token('{'),
        workflow,
        token('}'),
    ).parse_next(input)?;
    
    let span = input.span_from(start);
    
    Ok(WorkflowDef {
        name: name.into(),
        body,
        span,
    })
}
```

### Statement Parsers

```rust
/// Parse any workflow statement
pub fn workflow<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    
    let stmt = alt((
        observe_stmt,
        orient_stmt,
        propose_stmt,
        decide_stmt,
        check_stmt,
        act_stmt,
        oblige_stmt,
        let_stmt,
        if_stmt,
        for_stmt,
        par_stmt,
        with_stmt,
        maybe_stmt,
        must_stmt,
        attempt_stmt,
        retry_stmt,
        timeout_stmt,
        done_stmt,
    )).parse_next(input)?;
    
    // Handle optional continuation (semicolon + workflow)
    let result = if let Ok(_) = token(';').parse_next(input) {
        let cont = workflow.parse_next(input)?;
        let span = input.span_from(start);
        Workflow::Seq {
            first: Box::new(stmt),
            second: Box::new(cont),
            span,
        }
    } else {
        stmt
    };
    
    Ok(result)
}

/// observe <capability> (as <pattern>)? (then <workflow>)?
pub fn observe_stmt<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    
    keyword("observe").parse_next(input)?;
    let capability = capability_ref.parse_next(input)?;
    let binding = opt(preceded(keyword("as"), pattern)).parse_next(input)?;
    let continuation = opt(preceded(keyword("then"), workflow.map(Box::new))).parse_next(input)?;
    
    let span = input.span_from(start);
    
    Ok(Workflow::Observe {
        capability,
        binding,
        continuation,
        span,
    })
}

/// orient { <expr> } (as <pattern>)? (then <workflow>)?
pub fn orient_stmt<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    
    keyword("orient").parse_next(input)?;
    let expr = delimited(token('{'), expr, token('}')).parse_next(input)?;
    let binding = opt(preceded(keyword("as"), pattern)).parse_next(input)?;
    let continuation = opt(preceded(keyword("then"), workflow.map(Box::new))).parse_next(input)?;
    
    let span = input.span_from(start);
    
    Ok(Workflow::Orient {
        expr,
        binding,
        continuation,
        span,
    })
}

/// decide { <expr> } (under <policy>)? then <workflow> (else <workflow>)?
pub fn decide_stmt<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    
    keyword("decide").parse_next(input)?;
    let expr = delimited(token('{'), expr, token('}')).parse_next(input)?;
    let policy = opt(preceded(keyword("under"), identifier)).parse_next(input)?;
    keyword("then").parse_next(input)?;
    let then_branch = workflow.map(Box::new).parse_next(input)?;
    let else_branch = opt(preceded(keyword("else"), workflow.map(Box::new))).parse_next(input)?;
    
    let span = input.span_from(start);
    
    Ok(Workflow::Decide {
        expr,
        policy: policy.map(|s| s.into()),
        then_branch,
        else_branch,
        span,
    })
}

/// act <action> (where <guard>)?
pub fn act_stmt<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    
    keyword("act").parse_next(input)?;
    let action = action_ref.parse_next(input)?;
    let guard = opt(preceded(keyword("where"), guard)).parse_next(input)?;
    
    let span = input.span_from(start);
    
    Ok(Workflow::Act {
        action,
        guard,
        span,
    })
}

/// par { <workflow> (| <workflow>)* }
pub fn par_stmt<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    
    keyword("par").parse_next(input)?;
    let branches = delimited(
        token('{'),
        separated(1.., workflow, token('|')),
        token('}'),
    ).parse_next(input)?;
    
    let span = input.span_from(start);
    
    Ok(Workflow::Par { branches, span })
}

/// if <expr> then <workflow> (else <workflow>)?
pub fn if_stmt<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    
    keyword("if").parse_next(input)?;
    let condition = expr.parse_next(input)?;
    keyword("then").parse_next(input)?;
    let then_branch = workflow.map(Box::new).parse_next(input)?;
    let else_branch = opt(preceded(keyword("else"), workflow.map(Box::new))).parse_next(input)?;
    
    let span = input.span_from(start);
    
    Ok(Workflow::If {
        condition,
        then_branch,
        else_branch,
        span,
    })
}

/// for <pattern> in <expr> do <workflow>
pub fn for_stmt<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    
    keyword("for").parse_next(input)?;
    let pattern = pattern.parse_next(input)?;
    keyword("in").parse_next(input)?;
    let collection = expr.parse_next(input)?;
    keyword("do").parse_next(input)?;
    let body = workflow.map(Box::new).parse_next(input)?;
    
    let span = input.span_from(start);
    
    Ok(Workflow::For {
        pattern,
        collection,
        body,
        span,
    })
}

/// let <pattern> = <expr> (in <workflow>)?
pub fn let_stmt<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    
    keyword("let").parse_next(input)?;
    let pattern = pattern.parse_next(input)?;
    token('=').parse_next(input)?;
    let expr_val = expr.parse_next(input)?;
    let continuation = opt(preceded(keyword("in"), workflow.map(Box::new))).parse_next(input)?;
    
    let span = input.span_from(start);
    
    Ok(Workflow::Let {
        pattern,
        expr: expr_val,
        continuation,
        span,
    })
}

/// done
pub fn done_stmt<'a>(input: &mut ParseInput<'a>) -> PResult<Workflow, ParseError> {
    let start = *input;
    keyword("done").parse_next(input)?;
    let span = input.span_from(start);
    Ok(Workflow::Done { span })
}
```

### Supporting Parsers

```rust
/// Capability reference: identifier (with <args>)?
pub fn capability_ref<'a>(input: &mut ParseInput<'a>) -> PResult<CapabilityRef, ParseError> {
    let name = identifier.parse_next(input)?;
    let args = opt(preceded(
        keyword("with"),
        separated_list(0.., arg, token(',')),
    )).parse_next(input)?;
    
    Ok(CapabilityRef {
        name: name.into(),
        args: args.unwrap_or_default(),
    })
}

/// Action reference: identifier(<args>)
pub fn action_ref<'a>(input: &mut ParseInput<'a>) -> PResult<ActionRef, ParseError> {
    let name = identifier.parse_next(input)?;
    let args = delimited(
        token('('),
        separated_list(0.., expr, token(',')),
        token(')'),
    ).parse_next(input)?;
    
    Ok(ActionRef {
        name: name.into(),
        args,
    })
}
```

## TDD Steps

### Step 1: Implement workflow_def Parser

Start with the entry point parser for workflow definitions.

### Step 2: Implement Basic Statement Parsers

Implement observe, orient, act, done first as the simplest.

### Step 3: Implement Control Flow Parsers

Add if, for, par, seq handling.

### Step 4: Add Comprehensive Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_observe() {
        let input = r#"observe read_file with path: "/tmp/foo.txt" as content"#;
        let result = workflow.parse_peek(ParseInput::new(input));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_decide() {
        let input = r#"decide { x > 0 } then { act notify() } else { done }"#;
        let result = workflow.parse_peek(ParseInput::new(input));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_par() {
        let input = r#"par { act a() | act b() | act c() }"#;
        let result = workflow.parse_peek(ParseInput::new(input));
        assert!(result.is_ok());
        let (_, wf) = result.unwrap();
        assert!(matches!(wf, Workflow::Par { .. }));
    }
}
```

## Completion Checklist

- [ ] workflow_def parser
- [ ] All workflow statement parsers (observe, orient, propose, decide, check, act, oblige, let, if, for, par, with, maybe, must, attempt, retry, timeout, done)
- [ ] Sequential composition via semicolon
- [ ] Comprehensive unit tests for each parser
- [ ] Error recovery in workflow context
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Grammar coverage**: Does every workflow construct parse correctly?
2. **Associativity**: Is sequential composition (seq) right-associative?
3. **Error recovery**: Can we parse partially invalid workflows?

## Estimated Effort

6 hours

## Dependencies

- TASK-012: Parser core (uses combinators)
- TASK-014: Expression parser (expr is used in many statements)

## Blocked By

- TASK-012: Parser core

## Blocks

- TASK-014: Expression parser (mutual recursion)
- TASK-016: Lowering (needs parsed workflows)
