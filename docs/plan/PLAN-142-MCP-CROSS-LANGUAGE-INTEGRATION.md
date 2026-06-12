# PLAN-142: Ash-MCP Cross-Language Integration

## Phase: 142

## Status: 🔄 Planning (Not Started)

## Goal

Improve Ash-MCP precision for Ash programs and reduce latency for typical agent and editor tasks through bounded cross-language integration with Rust. Focus on high-value, low-complexity cross-language tools that provide 80% of the benefits with 20% of the complexity.

## Background

Phase 141 benchmark results revealed that ash-mCP excels at Ash-specific tasks (97% token reduction, perfect accuracy for T9 workflow primitives) but fails completely for Rust-heavy tasks (0% accuracy for T1-T6). The recommendation is to implement "bounded cross-language integration" - simple mappings between Ash and Rust symbols without building a complex cross-language type system.

This phase follows the project's "progressive implementation" strategy, starting with simple, high-value mappings that can be extended incrementally based on real usage feedback.

## Scope

### In scope

1. **Cross-language symbol mapping** - Simple Ash ↔ Rust symbol lookups
2. **Precision improvements** - Enable ash-mcp to find Rust implementations of Ash symbols
3. **Latency optimization** - Persistent daemon mode and caching to reduce 10-20x overhead
4. **Targeted agent/editor tools** - Focus on navigation and hover, not complex analysis

### Out of scope

- Full cross-language type checking (defer to future phases)
- Complex semantic relationships between Ash and Rust
- Deep Rust AST analysis (use rust-analyzer MCP for that)
- Editor incremental sync or live parsing

## Specification

- [MCP-BENCHMARK-RESULTS.md](../notes/MCP-BENCHMARK-RESULTS.md) — Phase 141 outcomes showing the need for cross-language support
- [SPEC-038: Rust LSP / MCP Research 2025](../spec/SPEC-038-RUST-LSP-MCP-RESEARCH-2025.md)
- [Plan cross-language integration approach](../docs/ash-mcp/cross-language-integration.md)

## Tasks

| Task | Description | Estimate | Status |
|------|-------------|----------|--------|
| [TASK-1420](tasks/TASK-1420-cross-lang-configuration.md) | Design cross-language configuration schema | 2h | 📋 Planned |
| [TASK-1421](tasks/TASK-1421-ash-to-rust-mapping.md) | Build Ash → Rust symbol mapping tools | 6h | 📋 Planned |
| [TASK-1422](tasks/TASK-1422-rust-to-ash-mapping.md) | Build Rust → Ash usage finder | 4h | 📋 Planned |
| [TASK-1423](tasks/TASK-1423-latency-optimization.md) | Implement persistent daemon and caching | 4h | 📋 Planned |
| [TASK-1424](tasks/TASK-1424-enhanced-hover.md) | Add Rust context to Ash hover | 3h | 📋 Planned |
| [TASK-1425](tasks/TASK-1425-verification-gates.md) | Unit, integration, and performance tests | 4h | 📋 Planned |
| [TASK-1426](tasks/TASK-1426-phase-evaluation.md) | Evaluate accuracy and latency improvements | 3h | 📋 Planned |

## Deliverable

- `ash-mcp` with cross-language navigation tools:
  - `ash_find_rust_implementation` - Go to definition for Ash symbols
  - `ash_find_ash_usage` - Find Ash usage of Rust symbols
  - `ash_hover_with_rust_context` - Enhanced hover with Rust information
- Performance improvements: <50ms latency for cross-language lookups
- Accuracy improvements: 80%+ for Ash-Rust mixed tasks (up from 0%)
- Benchmark report comparing pre/post integration metrics

## Timeline

~30 hours (3-5 weeks, part-time implementation).

## Risks

| Risk | Mitigation |
|------|------------|
| Configuration complexity | Start with simple YAML mappings, auto-generate where possible |
| Version coupling between Ash/Rust | Use heuristic mappings, graceful degradation when out of sync |
| Performance regression from cross-language lookups | Lazy initialization, caching, and measurable targets |
| Over-engineering complex relationships | Strictly bounded scope - simple symbol mappings only |

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-12 | Bounded cross-language integration only | Follows project's progressive implementation strategy; avoids architectural risks |
| 2026-06-12 | Focus on navigation tools first | Highest value for agents and editors; lowest complexity |
| 2026-06-12 | Graceful degradation requirement | Tools must work even when cross-language info is unavailable |
| 2026-06-12 | Performance targets before features | Latency optimization is critical for real-time usage |

## Notes

- This phase is based on the recommendation from Phase 141 benchmark results
- The approach follows the agreed "Option 1: Bounded Cross-Language Integration" strategy
- All tasks will be developed in dedicated git worktrees following the atomic task workflow
- This phase creates the foundation for potential Phase 143 expansion based on real usage feedback

---

**Phase Authority**: [Ash Implementation Plan](PLAN-INDEX.md)  
**Verification Baseline**: This document will be updated with git commit hash when phase starts  
**Related Work**: [Phase 140](PLAN-140-MCP-AGENT-INTELLIGENCE-SPIKE.md), [Phase 141](PLAN-141-MCP-BENCHMARK.md)