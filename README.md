# Battle Simulation

A complex agent-based simulation with genetic evolution, resource competition, and dynamic population dynamics.

## Features

- **Agent Evolution**: Agents have genes that affect speed, size, aggression, sense range, and energy efficiency
- **Resource Competition**: Agents compete for limited resources in a 2D environment
- **Genetic Reproduction**: Successful agents can reproduce, passing on their genes with mutations
- **Dynamic Population**: Population sizes fluctuate based on resource availability and competition
- **WebGL Rendering**: High-performance rendering with WebGL support and color-coded agents
- **Headless Mode**: Run simulations without a web interface for testing and optimization
- **Optimized Performance**: 10x faster simulation with minimal logging overhead
- **Parallel Processing**: Multi-core CPU utilization for complex calculations

## Quick Start

### Web Interface

To run the simulation in your browser:

1. Build the project:
   ```bash
   ./build.sh
   ```

2. Serve the files:
   ```bash
   python3 -m http.server 8000
   ```

3. Open `http://localhost:8000` in your browser

### Performance Features

- **10x Faster Simulation**: Optimized delta time and resource spawning
- **Clean Console**: Minimal logging for better performance
- **Color-Coded Agents**: Visual representation based on genes and energy
- **WebGL Acceleration**: Hardware-accelerated rendering when available
- **Canvas 2D Fallback**: Automatic fallback for WebGL-incompatible browsers

## Headless Simulation

The simulation can be run headlessly for testing and parameter optimization. This is useful for:

- Finding optimal parameters for stable, dynamic simulations
- Running long-term simulations without rendering overhead
- Automated testing and validation
- Parameter sweeps and optimization

### Using the Headless Simulation

The headless simulation is available through the library API. Here's an example:

```rust
use battleo::headless_simulation::{HeadlessSimulation, HeadlessSimulationConfig};

// Create a configuration
let config = HeadlessSimulationConfig {
    width: 1000.0,
    height: 800.0,
    initial_agents: 500,
    initial_resources: 500,
    target_duration_minutes: 5.0,
    ..Default::default()
};

// Run the simulation
let mut simulation = HeadlessSimulation::new(config);
let diagnostics = simulation.run();

// Analyze results
println!("Simulation completed in {:.2}s", diagnostics.duration_seconds);
println!("Final agents: {}", diagnostics.final_stats.agent_count);
println!("Stability score: {:.3}", diagnostics.stability_score);
println!("Is stable: {}", diagnostics.is_stable);
println!("Is dynamic: {}", diagnostics.is_dynamic);
```

### Test Harness

The test harness provides automated testing and parameter optimization:

```rust
use battleo::test_harness::TestHarness;
use battleo::headless_simulation::HeadlessSimulationConfig;

let mut harness = TestHarness::new();
harness.create_suite("my_test".to_string());

let config = HeadlessSimulationConfig::default();
let result = harness.run_single_test(config);

println!("Test score: {:.3}", result.score);
println!("Passed: {}", result.passed);
```

### Running Tests

The project includes comprehensive tests that demonstrate the headless functionality:

```bash
# Run all tests
cargo test

# Run specific tests
cargo test test_headless_simulation
cargo test test_harness_quick
cargo test test_comprehensive_simulation
cargo test test_long_simulation
cargo test test_parameter_optimization
```

### Configuration Options

The `HeadlessSimulationConfig` struct allows you to customize:

- **World Size**: `width` and `height` of the simulation area
- **Population**: `initial_agents` and `initial_resources` counts
- **Limits**: `max_agents` and `max_resources` to prevent explosion
- **Timing**: `target_duration_minutes` for simulation length
- **Stability**: `stability_threshold` for determining stability
- **Resource Spawning**: `resource_spawn_rate` for food availability

### Diagnostics

The simulation provides comprehensive diagnostics:

- **Population History**: Agent and resource counts over time
- **Energy Trends**: Total energy and fitness evolution
- **Stability Metrics**: Coefficient of variation and stability scores
- **Evolution Data**: Generation counts and reproduction statistics
- **Performance**: Execution time and steps per second

### Optimization

The test harness can automatically optimize parameters:

1. **Parameter Sweep**: Test different combinations of parameters
2. **Iterative Optimization**: Gradually improve parameters based on results
3. **Multi-Scenario Testing**: Test different simulation scenarios
4. **Scoring System**: Evaluate simulations based on stability, dynamics, and health

## Architecture

### Core Components

- **Agent**: Individual entities with genes, behavior, and lifecycle
- **Resource**: Food sources that agents consume
- **Simulation**: Main simulation loop and world management
- **Genes**: Genetic traits that affect agent behavior
- **HeadlessSimulation**: Non-rendering simulation for testing
- **TestHarness**: Automated testing and optimization framework
- **WebGlRenderer**: High-performance WebGL rendering with color coding

### Parallel Processing

The simulation uses Rayon for parallel processing:
- Agent updates are processed in parallel
- Resource updates are parallelized
- Complex calculations utilize all CPU cores
- Reproduction handling is parallelized

### Performance Optimizations

- **10x Speed Increase**: Optimized delta time (1/12s) and resource spawning
- **Reduced Logging**: Minimal console output for better performance
- **WebGL Rendering**: Hardware-accelerated graphics with color-coded agents
- **Parallel Processing**: Multi-core CPU utilization
- **Optimized Algorithms**: Efficient spatial queries and updates
- **Memory Management**: Minimal allocations during simulation
- **Smart Calculations**: Complex calculations only when beneficial

### Visual Features

- **Color-Coded Agents**: HSL color based on genes (speed, sense, size) and energy
- **Resource Visualization**: Color intensity based on energy content
- **WebGL Shaders**: Custom vertex and fragment shaders for efficient rendering
- **Canvas 2D Fallback**: Automatic fallback for WebGL-incompatible browsers

## Development

### Building

```bash
# Quick build script (recommended)
./build.sh

# Manual build
cargo build --release
wasm-pack build --target web
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_headless_simulation
cargo test test_parameter_optimization
```

### Dependencies

- **wasm-bindgen**: WebAssembly bindings
- **web-sys**: Web APIs for rendering
- **rand**: Random number generation
- **rayon**: Parallel processing
- **serde**: Serialization for diagnostics

## Example: Finding Stable Parameters

Here's how to find parameters for a stable, dynamic simulation:

```rust
use battleo::headless_simulation::HeadlessSimulationConfig;
use battleo::test_harness::TestHarness;

let mut harness = TestHarness::new();

// Test different configurations
for agents in [100, 200, 300, 400, 500] {
    for resources in [200, 300, 400, 500, 600] {
        let config = HeadlessSimulationConfig {
            initial_agents: agents,
            initial_resources: resources,
            target_duration_minutes: 5.0,
            ..Default::default()
        };
        
        let result = harness.run_single_test(config);
        if result.score > 0.8 {
            println!("Good config found: {} agents, {} resources, score: {:.3}", 
                     agents, resources, result.score);
        }
    }
}

harness.print_summary();
```

This will help you find parameters that create stable, dynamic simulations that last at least 5 minutes while remaining interesting and evolving.

## Recent Improvements

### Performance Enhancements
- **10x Faster Simulation**: Optimized timing and resource spawning
- **Reduced Logging**: Clean console output for better performance
- **Smart Calculations**: Complex calculations only when beneficial
- **Optimized Reproduction**: Better population control and energy management

### Visual Improvements
- **Color-Coded Agents**: Visual representation based on genetic traits
- **WebGL Optimization**: Hardware-accelerated rendering
- **Better Resource Visualization**: Energy-based color coding

### Stability Improvements
- **Population Control**: Better reproduction and death mechanics
- **Gene Stability**: Reduced mutation rates and better clamping
- **Resource Management**: Improved spawning and distribution
- **Energy Balance**: Better energy consumption and efficiency

## Troubleshooting

### Common Issues

1. **Port Already in Use**: If port 8000 is busy, use a different port:
   ```bash
   python3 -m http.server 8001
   ```

2. **WebGL Not Available**: The simulation automatically falls back to Canvas 2D rendering

3. **Performance Issues**: Reduce initial agent count in `src/simulation.rs` if needed

4. **Build Errors**: Ensure you have the latest Rust and wasm-pack installed

### Performance Tips

- Use WebGL rendering for best performance
- Monitor agent count - too many agents can slow down the simulation
- Close browser dev tools when not debugging
- Use headless mode for long-term testing
