# TASK-1423: Latency Optimization

**Estimate:** 4 hours  
**Status:** 📋 Planned  
**Phase:** 142  

## Description

Implement persistent daemon mode and caching to reduce ash-mcp's 10-20x latency overhead. Current ash-mcp takes 400-600ms per request vs 20-50ms for baseline grep. Target is <50ms for cached operations.

## Acceptance Criteria

✅ **All tests pass** - Daemon lifecycle, cache invalidation, performance  
✅ **Property tests extensive** - Concurrent access, cache eviction  
✅ **Code review** - Thread safety, performance critical paths  
✅ **Rust tooling** - Clean fmt, clippy, docs  
✅ **Documentation** - Daemon usage guide, performance benchmarks  

## Specifications

### Daemon Architecture

```rust
pub struct AshMcpDaemon {
    // Persistent state across requests
    config: CrossLangConfig,
    ast_cache: Arc<RwLock<AstCache>>,
    symbol_index: Arc<RwLock<SymbolIndex>>,
    file_watcher: RecommendedWatcher,
}

pub struct AstCache {
    // LRU cache of parsed ASTs
    entries: LinkedHashMap<PathBuf, AstEntry>,
    max_size: usize,
}

pub struct SymbolIndex {
    // Fast symbol lookup tables
    ash_to_rust: HashMap<String, RustSymbol>,
    rust_to_ash: HashMap<String, Vec<AshSymbol>>,
}
```

### Daemon Mode

```bash
# Run as persistent daemon
ash-mcp --daemon --workspace /path/to/workspace

# Daemon accepts multiple JSON-RPC requests on stdin
# Maintains state across requests
# Automatically exits on inactivity or SIGTERM
```

### Performance Targets

| Operation | Current | Target | Improvement |
|-----------|---------|--------|------------|
| First request | 500ms | 100ms | 5x faster |
| Cached request | 500ms | 10ms | 50x faster |
| Memory overhead | 50MB | 10MB | 5x reduction |
| Startup time | 300ms | 50ms | 6x faster |

## Implementation Steps

1. **Daemon mode infrastructure** (1h)
   - Add `--daemon` CLI flag
   - Implement JSON-RPC request loop
   - Add graceful shutdown handling

2. **AST caching** (1h)
   - Implement LRU cache for parsed files
   - Add file system watcher for cache invalidation
   - Thread-safe access patterns

3. **Symbol indexing** (1h)
   - Pre-build symbol lookup tables at startup
   - Incrementally update on file changes
   - Fast in-memory lookup structures

4. **Performance optimization** (1h)
   - Profile hot paths with perf/flamegraph
   - Optimize parsing bottlenecks
   - Implement connection pooling if needed

## Testing Strategy

```rust
#[tokio::test]
async fn test_daemon_lifecycle() {
    let daemon = AshMcpDaemon::new(&config).await.unwrap();
    
    // Test multiple requests
    for i in 0..10 {
        let result = daemon.handle_request(workspace_symbol_request()).await;
        assert!(result.is_ok());
    }
    
    // Test graceful shutdown
    daemon.shutdown().await;
}

#[tokio::test]
async fn test_cache_performance() {
    let cache = AstCache::new(100);
    let file = PathBuf::from("test.ash");
    
    // First parse (cache miss)
    let start = Instant::now();
    let ast1 = cache.get_or_parse(&file, || parse_file(&file)).await.unwrap();
    let miss_time = start.elapsed();
    
    // Second parse (cache hit)
    let start = Instant::now();
    let ast2 = cache.get_or_parse(&file, || parse_file(&file)).await.unwrap();
    let hit_time = start.elapsed();
    
    assert!(hit_time < miss_time / 10); // Cache hit should be 10x faster
    assert_eq!(ast1, ast2); // Same AST
}
```

## Verification

- Daemon starts/stops cleanly
- Cache invalidation works correctly
- Performance benchmarks meet targets
- No memory leaks or thread safety issues

## Dependencies

- TASK-1420: Cross-language configuration schema
- Previous ash-mcp infrastructure

---

**Task Authority**: [Phase 142](PLAN-142-MCP-CROSS-LANGUAGE-INTEGRATION.md)  
**Verification Baseline**: Will be updated with git commit when task starts