use wasm_bindgen::prelude::*;

pub mod agent;
pub mod ecs;
pub mod genes;
pub mod headless_simulation;
pub mod resource;
pub mod simulation_core;
pub mod web_simulation;
pub mod webgl_renderer;

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

        Ok(BattleSimulation { web_simulation })
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
    _closure: Option<Closure<dyn FnMut(JsValue)>>,
}

#[wasm_bindgen]
impl ParallelProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let worker_count = 1;
        Self {
            initialized: false,
            worker_count,
            _closure: None,
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
                // ecs_simulation::EcsSimulation::set_rayon_initialized(true); // This line was removed
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
                            // ecs_simulation::EcsSimulation::set_rayon_initialized(true); // This line was removed
                            web_sys::console::log_1(
                                &format!("Thread pool initialized with {} workers", worker_count)
                                    .into(),
                            );
                        }
                        None => {
                            // ecs_simulation::EcsSimulation::set_rayon_initialized(false); // This line was removed
                            web_sys::console::log_1(
                                &format!("Failed to initialize thread pool - SharedArrayBuffer may not be available").into(),
                            );
                        }
                    }
                }) as Box<dyn FnMut(JsValue)>);

                // Store the closure in the struct to prevent it from being dropped
                self._closure = Some(closure);

                // Get a reference to the stored closure
                let closure_ref = self._closure.as_ref().unwrap();

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
        // ecs_simulation::EcsSimulation::set_rayon_initialized(false); // This line was removed
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

    #[test]
    fn test_headless_simulation_v2() {
        use crate::headless_simulation::{HeadlessSimulationConfig, HeadlessSimulationV2};

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
        println!(
            "Final resources: {}",
            diagnostics.final_stats.resource_count
        );
        println!("Quality score: {:.3}", diagnostics.simulation_quality_score);
        println!("Steps per second: {:.1}", diagnostics.steps_per_second);

        // Basic assertions
        assert!(diagnostics.duration_seconds > 0.0);
        assert!(diagnostics.total_steps > 0);
        assert!(diagnostics.steps_per_second > 0.0);

        println!("Headless Simulation V2 test passed!");
    }
}
