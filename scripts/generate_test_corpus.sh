#!/bin/bash
set -e

# Generate test corpus for differential testing
# Usage: ./scripts/generate_test_corpus.sh --count 100 --output tests/differential/

COUNT=100
OUTPUT_DIR="tests/differential"
LEAN_REF="lean_reference/build/bin/ash_ref"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --count)
      COUNT="$2"
      shift 2
      ;;
    --output)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --lean)
      LEAN_REF="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

echo "Generating test corpus: $COUNT tests → $OUTPUT_DIR"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check if Lean reference is built
if [[ ! -f "$LEAN_REF" ]]; then
    echo "Error: Lean reference not found at $LEAN_REF"
    echo "Build with: cd lean_reference && lake build"
    exit 1
fi

# Generate test cases using Lean reference
echo "Generating test expressions..."
$LEAN_REF generate-tests --count "$COUNT" --output "$OUTPUT_DIR"

echo "Generated $(ls "$OUTPUT_DIR"/*.json 2>/dev/null | wc -l) test cases"
echo "Done!"
