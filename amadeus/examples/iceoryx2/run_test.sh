#!/bin/bash
# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

set -e

echo "=== Amadeus Iceoryx2 Python Test Runner ==="
echo

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is not installed or not in PATH"
    exit 1
fi

echo "✅ Python 3 found"

# Check if maturin is available
if ! command -v maturin &> /dev/null; then
    echo "⚠️  maturin not found, installing..."
    pip3 install maturin
fi

echo "✅ maturin available"

# Check if iceoryx2 Python binding is available
ICEORYX2_PYTHON_DIR="$PROJECT_ROOT/iceoryx2/iceoryx2-ffi/python"
if [ ! -d "$ICEORYX2_PYTHON_DIR" ]; then
    echo "❌ iceoryx2 Python binding directory not found at: $ICEORYX2_PYTHON_DIR"
    exit 1
fi

# Create virtual environment if it doesn't exist
if [ ! -d "$ICEORYX2_PYTHON_DIR/venv" ]; then
    echo "📦 Creating Python virtual environment..."
    cd "$ICEORYX2_PYTHON_DIR"
    python3 -m venv venv
fi

# Activate virtual environment and install iceoryx2
echo "🔧 Installing iceoryx2 Python binding..."
cd "$ICEORYX2_PYTHON_DIR"
source venv/bin/activate

# Check if iceoryx2 is already installed
if ! python3 -c "import iceoryx2" &> /dev/null; then
    echo "Building and installing iceoryx2 Python binding..."
    maturin develop --manifest-path Cargo.toml --target-dir ../../target/ff/python
else
    echo "✅ iceoryx2 Python binding already installed"
fi

echo "✅ Python 3 and iceoryx2 binding are ready"

    # Check if Rust project is built
if [ ! -f "$PROJECT_ROOT/amadeus/target/debug/amadeus" ]; then
    echo "⚠️  Rust binary not found, building..."
    cd "$PROJECT_ROOT/amadeus"
    cargo build
    cd "$SCRIPT_DIR"
fi

# Quick test first
echo
echo "🧪 Running quick verification test..."
cd "$SCRIPT_DIR"
python3 quick_test.py

echo
echo "Choose test mode:"
echo "1. Integration test (Python ↔ Python)"
echo "2. Publisher only"
echo "3. Subscriber only"
echo "4. Full system test (requires Rust dispatcher)"
echo "5. Quick verification only (already done above)"
read -p "Enter choice (1-5): " choice

case $choice in
    1)
        echo
        echo "🚀 Running Python integration test..."
        cd "$SCRIPT_DIR"
        python3 test_integration.py
        ;;
    2)
        echo
        echo "📤 Running Python publisher..."
        echo "Press Ctrl+C to stop"
        cd "$SCRIPT_DIR"
        python3 publisher.py
        ;;
    3)
        echo
        echo "👂 Running Python subscriber..."
        echo "Press Ctrl+C to stop"
        cd "$SCRIPT_DIR"
        python3 subscriber.py
        ;;
    4)
        echo
        echo "🔧 Full system test"
        echo
        echo "Step 1: Start Rust Amadeus dispatcher in another terminal:"
        echo "  cd $PROJECT_ROOT/amadeus && cargo run --example messaging"
        echo
        echo "Step 2: Run Python publisher in another terminal:"
        echo "  cd $SCRIPT_DIR && python3 publisher.py"
        echo
        echo "Step 3: Run Python subscriber in this terminal:"
        echo "  cd $SCRIPT_DIR && python3 subscriber.py"
        echo
        read -p "Press Enter when ready to start subscriber..."
        cd "$SCRIPT_DIR"
        python3 subscriber.py
        ;;
    5)
        echo
        echo "✅ Quick verification completed!"
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac
