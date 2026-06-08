# TASK-1375a: Fuel-Based Termination Analysis

## Status: 📝 Planned

## Description

Implement fuel-based termination checking for proof bodies.

## Requirements

1. Add `fuel` parameter to proof checking (default: 1000 steps)
2. Count reduction steps during proof normalization
3. Exceeding fuel = `untested` result (not error)

## Acceptance Criteria

- [ ] Fuel counter tracks reduction steps
- [ ] Fuel exceeded returns `untested`
- [ ] Configurable via CLI flag
- [ ] Test passes

## Related

- [TASK-1375](TASK-1375-stage3-totality-checking.md)
