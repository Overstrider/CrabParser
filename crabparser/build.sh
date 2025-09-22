#!/bin/bash

# Build script for CrabParser library

echo "Building CrabParser library..."

# Set cargo home
export CARGO_HOME="$HOME/.cargo"

# Check if we're in the virtual environment
if [ -z "$VIRTUAL_ENV" ]; then
    # Try to activate the virtual environment
    if [ -f "../venv/bin/activate" ]; then
        echo "Activating virtual environment..."
        source ../venv/bin/activate
    else
        echo "Error: Virtual environment not found. Please activate it first with 'source ../venv.sh'"
        exit 1
    fi
fi

# Check if maturin is installed
if ! command -v maturin &> /dev/null; then
    echo "Installing maturin..."
    pip install maturin
fi

# Build the library
echo "Building Rust extension..."
maturin develop --release

echo "Build complete! You can now import crabparser in Python."