# Battleo - Evolutionary Battle Simulation

A high-performance 2D browser simulation built with Rust and WebAssembly, featuring intelligent agents that battle for survival and evolve through natural selection. **Now with WebGL rendering and multicore CPU utilization for maximum performance on modern hardware.**

## Features

### 🧬 Genetic Evolution

- **Complex Gene System**: Agents have genes that affect speed, size, sense range, energy efficiency, aggression, and more
- **Inheritance & Mutation**: Offspring inherit traits from parents with realistic mutations
- **Natural Selection**: Successful agents reproduce, passing on beneficial traits
- **Learning Adaptation**: Agents can subtly adapt their behavior based on environmental conditions

### ⚔️ Battle Mechanics

- **Resource Competition**: Agents compete for growing resources with realistic energy dynamics
- **Predator-Prey Dynamics**: Larger agents can hunt smaller ones based on size ratios
- **Combat System**: Agents fight based on size, aggression, and energy levels
- **Survival Strategies**: Agents can flee from threats or hunt for prey
- **Environmental Stress**: Agents adapt to population density and resource scarcity

### 🎨 Beautiful Visualization

- **WebGL Rendering**: Hardware-accelerated graphics for smooth performance
- **Dynamic Colors**: Agent colors represent their genetic traits and energy levels
- **Smooth Animation**: 120 FPS rendering with optimized WebGL pipeline
- **Real-time Stats**: Live statistics showing population dynamics and evolution
- **Interactive UI**: Click to add agents or resources anywhere on the canvas

### ⚡ High Performance

- **WebAssembly**: Rust-compiled code for near-native performance
- **Multicore Processing**: Uses rayon for parallel agent updates across all CPU cores
- **WebGL Acceleration**: GPU-accelerated rendering for thousands of agents
- **Memory Efficient**: Smart data structures and automatic cleanup
- **Optimized for M1 Macs**: Takes advantage of unified memory architecture

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- A modern web browser with WebGL support

### Build & Run

1. **Clone and navigate to the project**

   ```bash
   cd battleo
   ```

2. **Build the WebAssembly module**

   ```bash
   ./build.sh
   ```

3. **Serve the application**

   ```bash
   # Using Python 3
   python3 -m http.server 8000

   # Or using Node.js
   npx serve .

   # Or using any other static file server
   ```

4. **Open in browser**
   Navigate to `http://localhost:8000`

## How to Play

### Controls

- **Start/Stop**: Control simulation playback
- **Step**: Advance simulation by one frame
- **Reset**: Clear and restart the simulation
- **Add Agent/Resource**: Click buttons then click on canvas to place

### Understanding the Simulation

#### Agent Traits

- **Speed**: How fast the agent moves (affects energy consumption)
- **Size**: Physical size (affects energy consumption and combat power)
- **Sense Range**: How far the agent can detect resources and other agents
- **Energy Efficiency**: How efficiently the agent uses energy
- **Aggression**: Likelihood to attack other agents
- **Reproduction Threshold**: Energy needed to reproduce
- **Mutation Rate**: How likely genes are to mutate in offspring

#### Agent Behavior

- **Seeking**: Looking for resources or potential prey/threats
- **Hunting**: Moving toward a target (resource or prey)
- **Feeding**: Consuming resources for energy
- **Fighting**: Engaging in combat with other agents
- **Fleeing**: Running from threats
- **Reproducing**: Creating offspring with inherited genes

#### Visual Indicators

- **Color**: Based on genetic traits (hue varies with speed and sense range)
- **Size**: Represents the agent's physical size
- **Direction Line**: Shows movement direction
- **Brightness**: Indicates energy levels
- **Resources**: Green circles that grow and regenerate

## Technical Details

### Architecture

- **Rust Backend**: Core simulation logic in Rust for maximum performance
- **WebAssembly**: Compiled to WASM for browser execution
- **WebGL Renderer**: Hardware-accelerated graphics rendering
- **Rayon**: Parallel processing for agent updates across CPU cores
- **Canvas 2D**: Fallback rendering for compatibility

### Performance Optimizations

- **Multicore CPU Utilization**: Parallel agent processing using rayon
- **WebGL Pipeline**: GPU-accelerated rendering for thousands of entities
- **Spatial Partitioning**: Efficient collision detection and targeting
- **Memory Pooling**: Reduced allocation overhead
- **Batch Rendering**: Optimized drawing operations
- **Complex Calculations**: Distributed across CPU cores for maximum utilization

### Genetic Algorithm

- **Fitness Function**: Combines multiple traits for overall fitness
- **Crossover**: Blends genes from both parents with realistic inheritance
- **Mutation**: Random changes to genes with configurable rates
- **Selection Pressure**: Natural selection through resource competition and combat
- **Learning Adaptation**: Subtle behavioral adaptation based on experience

### Recent Improvements

- **Enhanced Agent Survival**: Reduced energy consumption and increased resource value
- **Better Reproduction**: Lowered reproduction thresholds for sustainable populations
- **Optimized Performance**: Reduced computational overhead while maintaining complexity
- **WebGL Rendering**: Hardware acceleration for smooth visualization of large populations

## Development

### Project Structure

```
battleo/
├── src/
│   ├── lib.rs              # Main WASM bindings and WebGL renderer
│   ├── simulation.rs       # Core simulation logic with parallel processing
│   ├── agent.rs            # Agent behavior, genetics, and learning
│   ├── resource.rs         # Resource management and growth
│   ├── genes.rs            # Genetic trait system and inheritance
│   └── webgl_renderer.rs   # WebGL rendering pipeline
├── index.html              # Web interface
├── Cargo.toml              # Rust dependencies
├── build.sh                # Build script
└── README.md               # This file
```

### Adding Features

1. **New Genes**: Add to `genes.rs` and update inheritance logic
2. **New Behaviors**: Extend `AgentState` enum in `agent.rs`
3. **Visual Effects**: Modify WebGL rendering in `webgl_renderer.rs`
4. **UI Elements**: Update `index.html` and JavaScript

### Performance Tips

- **M1 Mac Optimization**: The simulation is optimized for Apple Silicon's unified memory
- **Agent Count**: Can handle 5000+ agents with WebGL acceleration
- **Resource Monitoring**: Monitor CPU/GPU usage in Activity Monitor
- **Debugging**: Use browser dev tools and `console.log` for debugging

## System Requirements

### Minimum

- Modern browser with WebGL support
- 4GB RAM
- Dual-core CPU

### Recommended

- Chrome 90+, Firefox 88+, Safari 14+
- 8GB+ RAM
- Multi-core CPU (4+ cores)
- Dedicated GPU
- **M1 Mac**: Optimized for Apple Silicon performance

## Browser Compatibility

- Chrome 90+ (WebGL 2.0)
- Firefox 88+ (WebGL 2.0)
- Safari 14+ (WebGL 2.0)
- Edge 90+ (WebGL 2.0)

## License

MIT License - feel free to use, modify, and distribute!

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly with different agent populations
5. Submit a pull request

## Acknowledgments

- Built with [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
- Parallel processing with [rayon](https://github.com/rayon-rs/rayon)
- WebGL rendering with [web-sys](https://github.com/rustwasm/wasm-bindgen/tree/main/crates/web-sys)
- Random number generation with [rand](https://github.com/rust-random/rand)
- Optimized for Apple Silicon and modern multicore systems

---

**Watch evolution in action with hardware-accelerated performance!** 🧬⚡🚀
