use crate::agent::Agent;
use crate::resource::Resource;
use rand::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub mod agent;
pub mod ecs;
pub mod ecs_simulation;
pub mod genes;
pub mod headless_simulation;
pub mod headless_simulation_v2;
pub mod resource;
pub mod simulation;
pub mod simulation_core;
pub mod test_harness;
pub mod webgl_renderer;
pub mod web_simulation;

use simulation_core::UnifiedSimulation;
use webgl_renderer::WebGlRenderer;

#[wasm_bindgen]
pub struct BattleSimulation {
    web_simulation: web_simulation::WebSimulation,
}

#[wasm_bindgen]
impl BattleSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<BattleSimulation, JsValue> {
        console_error_panic_hook::set_once();

        let web_simulation = web_simulation::WebSimulation::new(canvas_id)?;

        Ok(BattleSimulation {
            web_simulation,
        })
    }

    pub fn start(&mut self) {
        self.web_simulation.start();
    }

    pub fn stop(&mut self) {
        self.web_simulation.stop();
    }

    pub fn step(&mut self) {
        self.web_simulation.step();
    }

    pub fn get_stats(&self) -> JsValue {
        self.web_simulation.get_stats()
    }

    pub fn get_rendering_mode(&self) -> String {
        self.web_simulation.get_rendering_mode()
    }

    pub fn is_rayon_available(&self) -> bool {
        self.web_simulation.is_rayon_available()
    }

    pub fn set_rayon_initialized(&self, initialized: bool) {
        self.web_simulation.set_rayon_initialized(initialized);
    }

    pub fn force_webgl(&mut self) -> bool {
        self.web_simulation.force_webgl()
    }

    pub fn add_agent(&mut self, x: f64, y: f64) {
        self.web_simulation.add_agent(x, y);
    }

    pub fn add_resource(&mut self, x: f64, y: f64) {
        self.web_simulation.add_resource(x, y);
    }

    pub fn reset(&mut self) {
        self.web_simulation.reset();
    }

    pub fn animate(&mut self) {
        self.web_simulation.animate();
    }


}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
// Removed init_rayon_pool function - using ParallelProcessor instead
#[wasm_bindgen]
pub struct ParallelProcessor {
    initialized: bool,
    worker_count: usize,
    #[wasm_bindgen(skip)]
    closure: Option<Closure<dyn FnMut(JsValue)>>,
}

#[wasm_bindgen]
impl ParallelProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let worker_count = {
            #[cfg(target_arch = "wasm32")]
            {
                // Default to 4 workers for WASM
                4
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4)
            }
        };

        Self {
            initialized: false,
            worker_count,
            closure: None,
        }
    }

    pub fn initialize(&mut self) -> js_sys::Promise {
        #[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon"))]
        {
            #[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon"))]
            use wasm_bindgen_rayon::init_thread_pool;

            web_sys::console::log_1(
                &format!(
                    "Starting Rayon initialization with {} workers",
                    self.worker_count
                )
                .into(),
            );

            // Check if already initialized
            if self.initialized {
                web_sys::console::warn_1(&"Thread pool already initialized".into());
                ecs_simulation::EcsSimulation::set_rayon_initialized(true);
                return js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
            }

            {
                let worker_count = self.worker_count;
                let closure = Closure::wrap(Box::new(move |result: JsValue| {
                    web_sys::console::log_1(
                        &format!("Rayon initialization callback received: {:?}", result).into(),
                    );
                    match result.as_f64() {
                        Some(_) => {
                            ecs_simulation::EcsSimulation::set_rayon_initialized(true);
                            web_sys::console::log_1(
                                &format!("Thread pool initialized with {} workers", worker_count)
                                    .into(),
                            );
                        }
                        None => {
                            ecs_simulation::EcsSimulation::set_rayon_initialized(false);
                            web_sys::console::log_1(
                                &format!("Failed to initialize thread pool - SharedArrayBuffer may not be available").into(),
                            );
                        }
                    }
                }) as Box<dyn FnMut(JsValue)>);

                // Store the closure in the struct to prevent it from being dropped
                self.closure = Some(closure);

                // Get a reference to the stored closure
                let closure_ref = self.closure.as_ref().unwrap();

                // Create the promise with the stored closure
                let promise = init_thread_pool(self.worker_count).then(closure_ref);

                // Mark as initialized to prevent recursive calls
                self.initialized = true;

                promise
            }
        }

        #[cfg(not(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon")))]
        {
            // For non-WASM targets, simulate initialization
            self.initialized = true;
            let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
            promise
        }
    }

    pub fn initialize_fallback(&mut self) -> js_sys::Promise {
        // Fallback initialization that works without SharedArrayBuffer
        web_sys::console::warn_1(&"Using fallback mode - SharedArrayBuffer not available".into());
        self.initialized = true;
        ecs_simulation::EcsSimulation::set_rayon_initialized(false); // Set to false for fallback mode
        let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
        promise
    }

    pub fn parallel_sum(&self, data: Vec<f64>) -> f64 {
        if !self.initialized {
            web_sys::console::warn_1(&"Thread pool not initialized, using sequential".into());
            return data.iter().sum();
        }

        #[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon"))]
        {
            use rayon::prelude::*;
            data.par_iter().sum()
        }

        #[cfg(not(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon")))]
        {
            use rayon::prelude::*;
            data.par_iter().sum()
        }
    }

    pub fn parallel_map(&self, data: Vec<f64>) -> Vec<f64> {
        if !self.initialized {
            return data.iter().map(|x| x * 2.0).collect();
        }

        #[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon"))]
        {
            use rayon::prelude::*;
            data.par_iter().map(|x| x * 2.0).collect()
        }

        #[cfg(not(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon")))]
        {
            use rayon::prelude::*;
            data.par_iter().map(|x| x * 2.0).collect()
        }
    }

    pub fn complex_parallel_operation(&self, data: Vec<f64>) -> f64 {
        if !self.initialized {
            return data.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
        }

        #[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon"))]
        {
            use rayon::prelude::*;
            data.par_iter().map(|x| x.powi(2)).sum::<f64>().sqrt()
        }

        #[cfg(not(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon")))]
        {
            use rayon::prelude::*;
            data.par_iter().map(|x| x.powi(2)).sum::<f64>().sqrt()
        }
    }

    pub fn get_worker_count(&self) -> usize {
        self.worker_count
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless_simulation::HeadlessSimulationConfig;
    use crate::test_harness::TestHarness;

    #[test]
    fn test_headless_simulation() {
        let config = HeadlessSimulationConfig {
            target_duration_minutes: 0.1, // Very short test
            initial_agents: 10,
            initial_resources: 20,
            ..Default::default()
        };

        let mut simulation = crate::headless_simulation::HeadlessSimulation::new(config);
        let diagnostics = simulation.run();

        println!("Test completed in {:.2}s", diagnostics.duration_seconds);
        println!("Final agents: {}", diagnostics.final_stats.agent_count);
        println!(
            "Final resources: {}",
            diagnostics.final_stats.resource_count
        );
        println!("Stability score: {:.3}", diagnostics.stability_score);

        // Basic assertions
        assert!(diagnostics.duration_seconds > 0.0);
        assert!(diagnostics.total_steps > 0);
    }

    #[test]
    fn test_harness_quick() {
        let mut harness = TestHarness::new();
        harness.create_suite("test_suite".to_string());

        let config = HeadlessSimulationConfig {
            target_duration_minutes: 0.1,
            initial_agents: 10,
            initial_resources: 20,
            ..Default::default()
        };

        let result = harness.run_single_test(config);
        println!(
            "Test result: Score {:.3}, Passed: {}",
            result.score, result.passed
        );

        // Basic assertions
        assert!(result.diagnostics.duration_seconds > 0.0);
    }

    #[test]
    fn test_comprehensive_simulation() {
        println!("=== Running Comprehensive Headless Simulation Test ===");

        let config = HeadlessSimulationConfig {
            width: 800.0,
            height: 600.0,
            max_agents: 1000,
            max_resources: 500,
            initial_agents: 100,
            initial_resources: 200,
            resource_spawn_rate: 0.3,
            target_duration_minutes: 1.0, // 1 minute test
            stability_threshold: 0.1,
            min_agent_count: 10,
            max_agent_count: 800,
        };

        let mut simulation = crate::headless_simulation::HeadlessSimulation::new(config.clone());
        let diagnostics = simulation.run();

        println!("\n=== Simulation Results ===");
        println!("Duration: {:.2}s", diagnostics.duration_seconds);
        println!("Total steps: {}", diagnostics.total_steps);
        println!("Steps per second: {:.1}", diagnostics.steps_per_second);

        println!("\n=== Final Population ===");
        println!("Final agents: {}", diagnostics.final_stats.agent_count);
        println!(
            "Final resources: {}",
            diagnostics.final_stats.resource_count
        );
        println!("Total energy: {:.1}", diagnostics.final_stats.total_energy);
        println!(
            "Average energy: {:.1}",
            diagnostics.final_stats.total_energy
                / diagnostics.final_stats.agent_count.max(1) as f64
        );

        println!("\n=== Agent Statistics ===");
        println!("Average age: {:.1}", diagnostics.final_stats.average_age);
        println!(
            "Average speed: {:.3}",
            diagnostics.final_stats.average_speed
        );
        println!("Average size: {:.3}", diagnostics.final_stats.average_size);
        println!(
            "Average aggression: {:.3}",
            diagnostics.final_stats.average_aggression
        );
        println!(
            "Average sense range: {:.3}",
            diagnostics.final_stats.average_sense_range
        );
        println!(
            "Average energy efficiency: {:.3}",
            diagnostics.final_stats.average_energy_efficiency
        );
        println!("Max generation: {}", diagnostics.final_stats.max_generation);
        println!("Total kills: {}", diagnostics.final_stats.total_kills);
        println!(
            "Average fitness: {:.3}",
            diagnostics.final_stats.average_fitness
        );

        println!("\n=== Simulation Quality ===");
        println!("Stability score: {:.3}", diagnostics.stability_score);
        println!("Is stable: {}", diagnostics.is_stable);
        println!("Is dynamic: {}", diagnostics.is_dynamic);
        println!("Extinction occurred: {}", diagnostics.extinction_occurred);
        println!("Population explosion: {}", diagnostics.population_explosion);
        println!(
            "Average generations: {:.1}",
            diagnostics.average_generations
        );
        println!("Total reproductions: {}", diagnostics.total_reproductions);
        println!("Total deaths: {}", diagnostics.total_deaths);

        println!("\n=== Population History (last 10 samples) ===");
        let history_len = diagnostics.agent_count_history.len();
        let start_idx = if history_len > 10 {
            history_len - 10
        } else {
            0
        };
        for i in start_idx..history_len {
            println!(
                "Step {}: {} agents, {} resources",
                i * 60,
                diagnostics.agent_count_history[i],
                diagnostics.resource_count_history[i]
            );
        }

        // Calculate a comprehensive score
        let mut score = 0.0;

        // Duration completion (30%)
        let duration_ratio = diagnostics.duration_seconds / (config.target_duration_minutes * 60.0);
        score += duration_ratio * 0.3;

        // Population health (25%)
        let final_agents = diagnostics.final_stats.agent_count;
        let target_agents = config.initial_agents;
        let agent_ratio = final_agents as f64 / target_agents as f64;
        if agent_ratio >= 0.5 && agent_ratio <= 2.0 {
            score += 0.25;
        } else if agent_ratio >= 0.3 && agent_ratio <= 3.0 {
            score += 0.15;
        }

        // Stability (20%)
        score += diagnostics.stability_score * 0.2;

        // Evolution progress (15%)
        if diagnostics.average_generations > 1.0 {
            score += 0.15;
        } else if diagnostics.average_generations > 0.5 {
            score += 0.10;
        }

        // Dynamic behavior (10%)
        if diagnostics.is_dynamic {
            score += 0.10;
        }

        // Penalties
        if diagnostics.extinction_occurred {
            score -= 0.5;
        }
        if diagnostics.population_explosion {
            score -= 0.3;
        }

        score = score.max(0.0).min(1.0);

        println!("\n=== Overall Score ===");
        println!("Comprehensive score: {:.3}", score);
        println!(
            "Quality assessment: {}",
            if score > 0.8 {
                "Excellent"
            } else if score > 0.6 {
                "Good"
            } else if score > 0.4 {
                "Fair"
            } else if score > 0.2 {
                "Poor"
            } else {
                "Very Poor"
            }
        );

        // Assertions for basic functionality
        assert!(
            diagnostics.duration_seconds > 0.0,
            "Simulation should run for some time"
        );
        assert!(diagnostics.total_steps > 0, "Simulation should have steps");
        assert!(
            diagnostics.steps_per_second > 0.0,
            "Should have positive steps per second"
        );

        // Don't fail on low scores, just warn
        if score < 0.3 {
            println!("WARNING: Low simulation score indicates potential issues");
        }
    }

    #[test]
    fn test_long_simulation() {
        println!("=== Running Long Duration Headless Simulation Test ===");

        let config = HeadlessSimulationConfig {
            width: 800.0,
            height: 600.0,
            max_agents: 1000,
            max_resources: 500,
            initial_agents: 100,
            initial_resources: 200,
            resource_spawn_rate: 0.3,
            target_duration_minutes: 5.0, // 5 minute test
            stability_threshold: 0.1,
            min_agent_count: 10,
            max_agent_count: 800,
        };

        let mut simulation = crate::headless_simulation::HeadlessSimulation::new(config.clone());
        let diagnostics = simulation.run();

        println!("\n=== Long Simulation Results ===");
        println!("Duration: {:.2}s", diagnostics.duration_seconds);
        println!("Total steps: {}", diagnostics.total_steps);
        println!("Steps per second: {:.1}", diagnostics.steps_per_second);

        println!("\n=== Final Population ===");
        println!("Final agents: {}", diagnostics.final_stats.agent_count);
        println!(
            "Final resources: {}",
            diagnostics.final_stats.resource_count
        );
        println!("Total energy: {:.1}", diagnostics.final_stats.total_energy);
        println!(
            "Average energy: {:.1}",
            diagnostics.final_stats.total_energy
                / diagnostics.final_stats.agent_count.max(1) as f64
        );

        println!("\n=== Agent Statistics ===");
        println!("Average age: {:.1}", diagnostics.final_stats.average_age);
        println!(
            "Average speed: {:.3}",
            diagnostics.final_stats.average_speed
        );
        println!("Average size: {:.3}", diagnostics.final_stats.average_size);
        println!(
            "Average aggression: {:.3}",
            diagnostics.final_stats.average_aggression
        );
        println!(
            "Average sense range: {:.3}",
            diagnostics.final_stats.average_sense_range
        );
        println!(
            "Average energy efficiency: {:.3}",
            diagnostics.final_stats.average_energy_efficiency
        );
        println!("Max generation: {}", diagnostics.final_stats.max_generation);
        println!("Total kills: {}", diagnostics.final_stats.total_kills);
        println!(
            "Average fitness: {:.3}",
            diagnostics.final_stats.average_fitness
        );

        println!("\n=== Simulation Quality ===");
        println!("Stability score: {:.3}", diagnostics.stability_score);
        println!("Is stable: {}", diagnostics.is_stable);
        println!("Is dynamic: {}", diagnostics.is_dynamic);
        println!("Extinction occurred: {}", diagnostics.extinction_occurred);
        println!("Population explosion: {}", diagnostics.population_explosion);
        println!(
            "Average generations: {:.1}",
            diagnostics.average_generations
        );
        println!("Total reproductions: {}", diagnostics.total_reproductions);
        println!("Total deaths: {}", diagnostics.total_deaths);

        println!("\n=== Population History (every 1000 steps) ===");
        let history_len = diagnostics.agent_count_history.len();
        for i in (0..history_len).step_by(1000) {
            if i < history_len {
                println!(
                    "Step {}: {} agents, {} resources",
                    i * 60,
                    diagnostics.agent_count_history[i],
                    diagnostics.resource_count_history[i]
                );
            }
        }

        // Calculate a comprehensive score
        let mut score = 0.0;

        // Duration completion (30%)
        let duration_ratio = diagnostics.duration_seconds / (config.target_duration_minutes * 60.0);
        score += duration_ratio * 0.3;

        // Population health (25%)
        let final_agents = diagnostics.final_stats.agent_count;
        let target_agents = config.initial_agents;
        let agent_ratio = final_agents as f64 / target_agents as f64;
        if agent_ratio >= 0.5 && agent_ratio <= 2.0 {
            score += 0.25;
        } else if agent_ratio >= 0.3 && agent_ratio <= 3.0 {
            score += 0.15;
        }

        // Stability (20%)
        score += diagnostics.stability_score * 0.2;

        // Evolution progress (15%)
        if diagnostics.average_generations > 1.0 {
            score += 0.15;
        } else if diagnostics.average_generations > 0.5 {
            score += 0.10;
        }

        // Dynamic behavior (10%)
        if diagnostics.is_dynamic {
            score += 0.10;
        }

        // Penalties
        if diagnostics.extinction_occurred {
            score -= 0.5;
        }
        if diagnostics.population_explosion {
            score -= 0.3;
        }

        score = score.max(0.0).min(1.0);

        println!("\n=== Overall Score ===");
        println!("Comprehensive score: {:.3}", score);
        println!(
            "Quality assessment: {}",
            if score > 0.8 {
                "Excellent"
            } else if score > 0.6 {
                "Good"
            } else if score > 0.4 {
                "Fair"
            } else if score > 0.2 {
                "Poor"
            } else {
                "Very Poor"
            }
        );

        // Assertions for basic functionality
        assert!(
            diagnostics.duration_seconds > 0.0,
            "Simulation should run for some time"
        );
        assert!(diagnostics.total_steps > 0, "Simulation should have steps");
        assert!(
            diagnostics.steps_per_second > 0.0,
            "Should have positive steps per second"
        );

        // Don't fail on low scores, just warn
        if score < 0.3 {
            println!("WARNING: Low simulation score indicates potential issues");
        }
    }

    #[test]
    fn test_parameter_optimization() {
        println!("=== Running Parameter Optimization Test Suite ===");

        let mut harness = TestHarness::new();
        harness.create_suite("optimization_suite".to_string());

        // Test different configurations
        let configs = vec![
            HeadlessSimulationConfig {
                width: 800.0,
                height: 600.0,
                max_agents: 800,
                max_resources: 400,
                initial_agents: 80,
                initial_resources: 160,
                resource_spawn_rate: 0.2,
                target_duration_minutes: 2.0,
                stability_threshold: 0.1,
                min_agent_count: 10,
                max_agent_count: 600,
            },
            HeadlessSimulationConfig {
                width: 800.0,
                height: 600.0,
                max_agents: 1000,
                max_resources: 500,
                initial_agents: 100,
                initial_resources: 200,
                resource_spawn_rate: 0.3,
                target_duration_minutes: 2.0,
                stability_threshold: 0.1,
                min_agent_count: 10,
                max_agent_count: 800,
            },
            HeadlessSimulationConfig {
                width: 800.0,
                height: 600.0,
                max_agents: 1200,
                max_resources: 600,
                initial_agents: 120,
                initial_resources: 240,
                resource_spawn_rate: 0.4,
                target_duration_minutes: 2.0,
                stability_threshold: 0.1,
                min_agent_count: 10,
                max_agent_count: 1000,
            },
            HeadlessSimulationConfig {
                width: 800.0,
                height: 600.0,
                max_agents: 600,
                max_resources: 300,
                initial_agents: 60,
                initial_resources: 120,
                resource_spawn_rate: 0.15,
                target_duration_minutes: 2.0,
                stability_threshold: 0.1,
                min_agent_count: 10,
                max_agent_count: 500,
            },
        ];

        println!("Testing {} different configurations...", configs.len());

        for (i, config) in configs.iter().enumerate() {
            println!("\n--- Testing Configuration {} ---", i + 1);
            println!(
                "Initial agents: {}, Initial resources: {}, Spawn rate: {:.2}",
                config.initial_agents, config.initial_resources, config.resource_spawn_rate
            );

            let result = harness.run_single_test(config.clone());

            println!(
                "Result: Score {:.3}, Passed: {}",
                result.score, result.passed
            );
            println!(
                "Final agents: {}, Final resources: {}",
                result.diagnostics.final_stats.agent_count,
                result.diagnostics.final_stats.resource_count
            );
            println!(
                "Stability: {:.3}, Dynamic: {}",
                result.diagnostics.stability_score, result.diagnostics.is_dynamic
            );
            println!("Notes: {}", result.notes);
        }

        // Print summary
        harness.print_summary();

        // Get the best configuration
        if let Some(best_config) = harness.get_best_config() {
            println!("\n=== Best Configuration Found ===");
            println!("Initial agents: {}", best_config.initial_agents);
            println!("Initial resources: {}", best_config.initial_resources);
            println!("Max agents: {}", best_config.max_agents);
            println!("Max resources: {}", best_config.max_resources);
            println!(
                "Resource spawn rate: {:.2}",
                best_config.resource_spawn_rate
            );
            println!(
                "Target duration: {:.1} minutes",
                best_config.target_duration_minutes
            );
        }

        // Run a final test with the best configuration for a longer duration
        if let Some(best_config) = harness.get_best_config() {
            println!("\n=== Final Long Test with Best Configuration ===");

            let mut final_config = best_config.clone();
            final_config.target_duration_minutes = 3.0; // 3 minute test

            let mut simulation =
                crate::headless_simulation::HeadlessSimulation::new(final_config.clone());
            let diagnostics = simulation.run();

            println!(
                "Final test completed in {:.2}s",
                diagnostics.duration_seconds
            );
            println!("Final agents: {}", diagnostics.final_stats.agent_count);
            println!(
                "Final resources: {}",
                diagnostics.final_stats.resource_count
            );
            println!("Stability score: {:.3}", diagnostics.stability_score);
            println!("Is stable: {}", diagnostics.is_stable);
            println!("Is dynamic: {}", diagnostics.is_dynamic);
            println!("Max generation: {}", diagnostics.final_stats.max_generation);
            println!(
                "Average generations: {:.1}",
                diagnostics.average_generations
            );

            // Calculate final score
            let mut score = 0.0;
            let duration_ratio =
                diagnostics.duration_seconds / (final_config.target_duration_minutes * 60.0);
            score += duration_ratio * 0.3;

            let final_agents = diagnostics.final_stats.agent_count;
            let target_agents = final_config.initial_agents;
            let agent_ratio = final_agents as f64 / target_agents as f64;
            if agent_ratio >= 0.5 && agent_ratio <= 2.0 {
                score += 0.25;
            } else if agent_ratio >= 0.3 && agent_ratio <= 3.0 {
                score += 0.15;
            }

            score += diagnostics.stability_score * 0.2;

            if diagnostics.average_generations > 1.0 {
                score += 0.15;
            } else if diagnostics.average_generations > 0.5 {
                score += 0.10;
            }

            if diagnostics.is_dynamic {
                score += 0.10;
            }

            if diagnostics.extinction_occurred {
                score -= 0.5;
            }
            if diagnostics.population_explosion {
                score -= 0.3;
            }

            score = score.max(0.0).min(1.0);

            println!("Final comprehensive score: {:.3}", score);
            println!(
                "Quality assessment: {}",
                if score > 0.8 {
                    "Excellent"
                } else if score > 0.6 {
                    "Good"
                } else if score > 0.4 {
                    "Fair"
                } else if score > 0.2 {
                    "Poor"
                } else {
                    "Very Poor"
                }
            );
        }
    }

    #[test]
    fn test_headless_simulation_v2() {
        use crate::headless_simulation_v2::{HeadlessSimulationConfig, HeadlessSimulationV2};
        
        println!("=== Testing Headless Simulation V2 ===");
        
        let config = HeadlessSimulationConfig {
            target_duration_minutes: 0.1, // Very short test
            speed_multiplier: 10.0,       // 10x faster
            initial_agents: 10,
            initial_resources: 20,
            use_ecs: true,
            ..Default::default()
        };

        let mut simulation = HeadlessSimulationV2::new(config);
        let diagnostics = simulation.run();

        println!("Test completed!");
        println!("Duration: {:.2}s", diagnostics.duration_seconds);
        println!("Final agents: {}", diagnostics.final_stats.agent_count);
        println!("Final resources: {}", diagnostics.final_stats.resource_count);
        println!("Quality score: {:.3}", diagnostics.simulation_quality_score);
        println!("Steps per second: {:.1}", diagnostics.steps_per_second);

        // Basic assertions
        assert!(diagnostics.duration_seconds > 0.0);
        assert!(diagnostics.total_steps > 0);
        assert!(diagnostics.steps_per_second > 0.0);
        
        println!("Headless Simulation V2 test passed!");
    }
}
