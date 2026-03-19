#!/bin/bash
set -e

# Differential testing between Lean and Rust implementations
# Usage: ./scripts/differential_test.sh [--lean PATH] [--rust PATH] [--corpus PATH] [--count N]

LEAN_BIN="lean_reference/build/bin/ash_ref"
RUST_BIN="target/release/ash"
CORPUS_DIR="tests/differential"
COUNT=100
PASSED=0
FAILED=0

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --lean)
      LEAN_BIN="$2"
      shift 2
      ;;
    --rust)
      RUST_BIN="$2"
      shift 2
      ;;
    --corpus)
      CORPUS_DIR="$2"
      shift 2
      ;;
    --count)
      COUNT="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

echo "======================================"
echo "  Differential Testing"
echo "======================================"
echo "Lean: $LEAN_BIN"
echo "Rust: $RUST_BIN"
echo "Corpus: $CORPUS_DIR"
echo "Count: $COUNT"
echo ""

# Check binaries exist
if [[ ! -f "$LEAN_BIN" ]]; then
    echo "Error: Lean binary not found at $LEAN_BIN"
    exit 1
fi

if [[ ! -f "$RUST_BIN" ]]; then
    echo "Error: Rust binary not found at $RUST_BIN"
    exit 1
fi

# Generate test corpus if needed
if [[ ! -d "$CORPUS_DIR" ]] || [[ $(ls "$CORPUS_DIR"/*.json 2>/dev/null | wc -l) -lt $COUNT ]]; then
    echo "Generating test corpus..."
    mkdir -p "$CORPUS_DIR"
    ./scripts/generate_test_corpus.sh --lean "$LEAN_BIN" --count "$COUNT" --output "$CORPUS_DIR"
fi

# Run differential tests
echo "Running differential tests..."
echo ""

for test_file in "$CORPUS_DIR"/*.json; do
    if [[ ! -f "$test_file" ]]; then
        continue
    fi
    
    test_name=$(basename "$test_file")
    
    # Run Lean interpreter
    lean_result=$("$LEAN_BIN" eval "$test_file" 2>/dev/null || echo '{"status":"error","error":"lean_crash"}')
    
    # Run Rust interpreter
    rust_result=$("$RUST_BIN" eval "$test_file" 2>/dev/null || echo '{"status":"error","error":"rust_crash"}')
    
    # Compare results (simplified - actual comparison would be more sophisticated)
    if [[ "$lean_result" == "$rust_result" ]]; then
        echo "✓ $test_name"
        PASSED=$((PASSED + 1))
    else
        echo "✗ $test_name"
        echo "  Lean: $lean_result"
        echo "  Rust: $rust_result"
        FAILED=$((FAILED + 1))
        
        # Save failing test for analysis
        mkdir -p "$CORPUS_DIR/failures"
        cp "$test_file" "$CORPUS_DIR/failures/$test_name"
        echo "$lean_result" > "$CORPUS_DIR/failures/${test_name%.json}.lean.json"
        echo "$rust_result" > "$CORPUS_DIR/failures/${test_name%.json}.rust.json"
    fi
done

echo ""
echo "======================================"
echo "Results: $PASSED passed, $FAILED failed"
echo "======================================"

if [[ $FAILED -gt 0 ]]; then
    echo "Failing tests saved to: $CORPUS_DIR/failures/"
    exit 1
else
    echo "All tests passed! ✓"
    exit 0
fi
