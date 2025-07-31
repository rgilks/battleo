# BattleO Unified Architecture

This document describes the refactored architecture that supports both headless simulation for high-speed evaluation and web browser display using the same core simulation code.

## Overview

The new architecture provides:

1. **Unified Simulation Core** - Same simulation logic for both headless and web modes
2. **High-Speed Headless Evaluation** - Run simulations at 10x+ speed for tuning and testing
3. **Web Browser Display** - Full WebGL rendering with real-time visualization
4. **CLI Interface** - Command-line tool for batch processing and automation

## Architecture Components

### Core Simulation (`simulation_core.rs`)

The heart of the system with three main components:

#### `SimulationEngine` Trait
- Defines the interface for simulation engines
- Allows pluggable implementations (ECS vs Legacy)

#### `EcsSimulationEngine`
- Uses the Entity Component System for high performance
- Supports parallel processing with Rayon
- Better for large-scale simulations

#### `LegacySimulationEngine`
- Traditional object-oriented approach
- Simpler implementation, easier to debug
- Good for smaller simulations or testing

#### `UnifiedSimulation`
- Wrapper that chooses between ECS and Legacy engines
- Provides a consistent interface regardless of backend
- Handles configuration and engine selection

### Headless Simulation (`headless_simulation_v2.rs`)

Optimized for high-speed evaluation:

- **Speed Multiplier**: Run simulations 10x+ faster than real-time
- **Progress Reporting**: Real-time feedback on simulation progress
- **Early Termination**: Stop on extinction, explosion, or collapse
- **Comprehensive Diagnostics**: Detailed metrics and quality scoring
- **Batch Processing**: Run multiple simulations with different parameters

### Web Simulation (`web_simulation.rs`)

Optimized for real-time visualization:

- **WebGL Rendering**: Hardware-accelerated graphics
- **Canvas 2D Fallback**: Works on all browsers
- **Real-time Stats**: Live population and performance metrics
- **Interactive Controls**: Add agents, resources, reset simulation

### CLI Interface (`src/bin/headless_runner.rs`)

Command-line tool for automation:

```bash
# Basic usage
cargo run --bin headless_runner -- --duration 5.0 --agents 100 --resources 200

# High-speed evaluation
cargo run --bin headless_runner -- --duration 10.0 --speed 20.0 --engine ecs

# Save results to file
cargo run --bin headless_runner -- --duration 2.0 --output results.json

# Verbose output
cargo run --bin headless_runner -- --duration 1.0 --verbose
```

## Usage Examples

### Headless High-Speed Evaluation

```rust
use battleo::headless_simulation_v2::{HeadlessSimulationConfig, HeadlessSimulationV2};

let config = HeadlessSimulationConfig {
    target_duration_minutes: 10.0,
    speed_multiplier: 20.0,  // 20x faster than real-time
    use_ecs: true,           // Use ECS for better performance
    initial_agents: 500,
    initial_resources: 500,
    ..Default::default()
};

let mut simulation = HeadlessSimulationV2::new(config);
let diagnostics = simulation.run();

println!("Quality score: {:.3}", diagnostics.simulation_quality_score);
println!("Steps per second: {:.1}", diagnostics.steps_per_second);
```

### Web Browser Display

```rust
use battleo::web_simulation::WebSimulation;

let mut web_sim = WebSimulation::new("canvas")?;
web_sim.start();  // Starts real-time animation loop
```

### Parameter Optimization

```bash
# Test different configurations
for agents in 50 100 200; do
    for resources in 100 200 400; do
        cargo run --bin headless_runner -- \
            --duration 2.0 \
            --agents $agents \
            --resources $resources \
            --speed 10.0 \
            --output "results_${agents}_${resources}.json"
    done
done
```

## Performance Characteristics

### Headless Mode
- **Speed**: 10-50x faster than real-time
- **Memory**: Optimized for batch processing
- **CPU**: Can utilize all cores with Rayon
- **Use Case**: Parameter tuning, batch evaluation, research

### Web Mode
- **Speed**: Real-time (60 FPS target)
- **Memory**: Optimized for rendering
- **GPU**: WebGL acceleration when available
- **Use Case**: Interactive visualization, debugging, demos

## Configuration

Both modes use the same `SimulationConfig` structure:

```rust
pub struct SimulationConfig {
    pub width: f64,
    pub height: f64,
    pub max_agents: usize,
    pub max_resources: usize,
    pub initial_agents: usize,
    pub initial_resources: usize,
    pub resource_spawn_rate: f64,
    pub target_duration_minutes: f64,
    pub stability_threshold: f64,
    pub min_agent_count: usize,
    pub max_agent_count: usize,
    pub use_ecs: bool,
}
```

## Migration from Old Code

The old `BattleSimulation` struct now wraps `WebSimulation` for backward compatibility. The core simulation logic has been extracted and unified, so both headless and web modes use identical simulation behavior.

## Future Enhancements

1. **Distributed Processing**: Run simulations across multiple machines
2. **Machine Learning Integration**: Use simulation results for AI training
3. **Advanced Visualization**: 3D rendering, heat maps, statistics charts
4. **Real-time Collaboration**: Multiple users viewing the same simulation
5. **Plugin System**: Custom agent behaviors and rendering effects 