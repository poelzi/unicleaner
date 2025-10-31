#!/usr/bin/env bash
set -euo pipefail

# Script to verify that the musl binary is truly static with no dynamic dependencies

BINARY="${1:-result/bin/unicleaner}"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

echo "=== Verifying static binary: $BINARY ==="
echo

echo "1. File type:"
file "$BINARY"
echo

echo "2. Checking for dynamic dependencies:"
if ldd "$BINARY" 2>&1 | grep -q "not a dynamic executable"; then
    echo "✅ PASS: Binary is statically linked (no dynamic dependencies)"
else
    echo "❌ FAIL: Binary has dynamic dependencies:"
    ldd "$BINARY"
    exit 1
fi
echo

echo "3. Binary size:"
ls -lh "$BINARY" | awk '{print $5, $9}'
echo

echo "4. Running readelf to check for interpreter:"
if readelf -l "$BINARY" 2>/dev/null | grep -q "INTERP"; then
    echo "❌ FAIL: Binary has dynamic interpreter:"
    readelf -l "$BINARY" | grep -A2 "INTERP"
    exit 1
else
    echo "✅ PASS: No dynamic interpreter (fully static)"
fi
echo

echo "5. Quick smoke test:"
if "$BINARY" --version > /dev/null 2>&1; then
    echo "✅ PASS: Binary executes successfully"
    "$BINARY" --version
else
    echo "❌ FAIL: Binary failed to execute"
    exit 1
fi
echo

echo "=== All checks passed! Binary is truly static. ==="
