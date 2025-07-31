#!/bin/bash

echo "Building headless runner..."
cargo build --bin headless_runner

if [ $? -eq 0 ]; then
    echo "Build successful! Testing headless runner..."
    
    # Test with default configuration
    echo "Running with default configuration..."
    ./target/debug/headless_runner --duration 0.1 --agents 10 --resources 20 --speed 5.0 --verbose
    
    echo ""
    echo "Running with ECS engine..."
    ./target/debug/headless_runner --duration 0.1 --agents 10 --resources 20 --speed 5.0 --engine ecs --verbose
    
    echo ""
    echo "Running with legacy engine..."
    ./target/debug/headless_runner --duration 0.1 --agents 10 --resources 20 --speed 5.0 --engine legacy --verbose
    
    echo ""
    echo "All tests completed!"
else
    echo "Build failed!"
    exit 1
fi 