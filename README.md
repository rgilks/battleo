# BattleO - Unified Simulation Framework

A high-performance agent-based simulation framework that supports both headless execution for tuning/evaluation and web browser visualization using the same core simulation logic.

## 🚀 Features

- **Unified Simulation Core**: Single codebase for both headless and web modes
- **High-Speed Headless Mode**: Run simulations without graphics for parameter tuning and evaluation
- **Web Browser Visualization**: Real-time WebGL/Canvas2D rendering in browsers
- **Dual Engine Support**: ECS (Entity Component System) and Legacy simulation engines
- **Performance Optimization**: Speed multipliers for rapid iteration
- **Comprehensive Diagnostics**: Detailed metrics and quality assessment

## 🏗️ Architecture

The simulation is built around a unified core with three main components:

### 1. Simulation Core (`simulation_core.rs`)

- `SimulationEngine` trait for engine abstraction
- `EcsSimulationEngine`: High-performance ECS-based simulation
- `LegacySimulationEngine`: Traditional vector-based simulation
- `UnifiedSimulation`: Dynamic engine selection wrapper

### 2. Headless Simulation (`headless_simulation_v2.rs`)

- `HeadlessSimulationV2`: High-speed, non-graphical simulation runner
- Configurable speed multipliers for rapid iteration
- Comprehensive diagnostics and quality metrics
- Early termination and progress reporting

### 3. Web Simulation (`web_simulation.rs`)

- `WebSimulation`: Browser-based simulation with rendering
- WebGL and Canvas2D rendering support
- Real-time visualization and interaction
- Same simulation core as headless mode

## 📦 Installation

```bash
git clone <repository>
cd battleo
cargo build --release
```

## 🎯 Usage

### Headless Mode (High-Speed Tuning)

```rust
use battleo::headless_simulation_v2::{HeadlessSimulationConfig, HeadlessSimulationV2};

// Configure simulation
let config = HeadlessSimulationConfig {
    target_duration_minutes: 2.0,    // Run for 2 minutes
    speed_multiplier: 20.0,          // 20x faster than real-time
    initial_agents: 100,
    initial_resources: 200,
    use_ecs: true,                   // Use ECS engine
    ..Default::default()
};

// Run simulation
let mut simulation = HeadlessSimulationV2::new(config);
let diagnostics = simulation.run();

// Analyze results
println!("Duration: {:.2}s", diagnostics.duration_seconds);
println!("Steps per second: {:.1}", diagnostics.steps_per_second);
println!("Quality score: {:.3}", diagnostics.simulation_quality_score);
println!("Final agents: {}", diagnostics.final_stats.agent_count);
```

### Web Mode (Browser Visualization)

```javascript
// Initialize simulation
const simulation = new BattleSimulation("canvas-id");

// Start visualization
simulation.start();

// Add agents/resources
simulation.add_agent(100, 100);
simulation.add_resource(200, 200);

// Get statistics
const stats = simulation.get_stats();
console.log("Agent count:", stats.agent_count);
console.log("Resource count:", stats.resource_count);
```

### Performance Comparison

```rust
// Test ECS vs Legacy performance
let config_ecs = HeadlessSimulationConfig {
    use_ecs: true,
    speed_multiplier: 10.0,
    ..Default::default()
};

let config_legacy = HeadlessSimulationConfig {
    use_ecs: false,
    speed_multiplier: 10.0,
    ..Default::default()
};

let mut sim_ecs = HeadlessSimulationV2::new(config_ecs);
let mut sim_legacy = HeadlessSimulationV2::new(config_legacy);

let diagnostics_ecs = sim_ecs.run();
let diagnostics_legacy = sim_legacy.run();

println!("ECS performance: {:.1} steps/sec", diagnostics_ecs.steps_per_second);
println!("Legacy performance: {:.1} steps/sec", diagnostics_legacy.steps_per_second);
```

## 🔧 Configuration

### HeadlessSimulationConfig

| Parameter                 | Type  | Default | Description                           |
| ------------------------- | ----- | ------- | ------------------------------------- |
| `target_duration_minutes` | f64   | 1.0     | Target simulation duration            |
| `speed_multiplier`        | f64   | 1.0     | Speed multiplier for faster execution |
| `initial_agents`          | usize | 50      | Initial number of agents              |
| `initial_resources`       | usize | 100     | Initial number of resources           |
| `use_ecs`                 | bool  | true    | Use ECS engine (false for legacy)     |
| `width`                   | f64   | 800.0   | Simulation world width                |
| `height`                  | f64   | 600.0   | Simulation world height               |
| `max_agents`              | usize | 1000    | Maximum agents allowed                |
| `max_resources`           | usize | 500     | Maximum resources allowed             |

### SimulationDiagnostics

The headless simulation provides comprehensive diagnostics:

- **Performance**: Duration, steps per second, total steps
- **Population**: Final agent/resource counts, energy levels
- **Quality**: Stability score, extinction detection, population explosion
- **Evolution**: Generations, reproductions, deaths, fitness metrics

## 🚀 Performance

### Headless Mode

- **Speed**: 10-100x faster than real-time with speed multipliers
- **Memory**: Optimized for large-scale simulations
- **Scalability**: Supports thousands of agents efficiently

### Web Mode

- **Rendering**: 60 FPS WebGL/Canvas2D rendering
- **Interactivity**: Real-time agent/resource addition
- **Compatibility**: Works in all modern browsers

## 🔄 Workflow

1. **Tune Parameters**: Use headless mode to rapidly test configurations
2. **Evaluate Performance**: Analyze diagnostics and quality metrics
3. **Visualize Results**: Use web mode to see the simulation in action
4. **Iterate**: Refine parameters based on both metrics and visualization

## 📊 Example Results

```
=== Headless Simulation Results ===
Duration: 1.23s
Steps per second: 1,847.3
Quality score: 0.892
Final agents: 87
Final resources: 156
Stability score: 0.85
Is stable: true
Is dynamic: true
Max generation: 5
Total reproductions: 23
Total deaths: 12
```

## 🛠️ Development

### Building

```bash
# Build library
cargo build --release

# Build WASM for web
wasm-pack build --target web
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_headless_simulation_v2
```

## 📝 License

[Add your license here]

## 🤝 Contributing

[Add contribution guidelines here]
