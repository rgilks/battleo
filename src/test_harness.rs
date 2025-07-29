use crate::headless_simulation::{HeadlessSimulation, HeadlessSimulationConfig, SimulationDiagnostics};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;
use rand::Rng;

#[derive(Clone, Serialize)]
pub struct TestResult {
    pub config: HeadlessSimulationConfig,
    pub diagnostics: SimulationDiagnostics,
    pub score: f64,
    pub passed: bool,
    pub notes: String,
}

#[derive(Clone, Serialize)]
pub struct TestSuite {
    pub name: String,
    pub results: Vec<TestResult>,
    pub best_config: Option<HeadlessSimulationConfig>,
    pub best_score: f64,
    pub total_runs: usize,
    pub successful_runs: usize,
    pub average_duration: f64,
}

impl TestSuite {
    pub fn new(name: String) -> Self {
        Self {
            name,
            results: Vec::new(),
            best_config: None,
            best_score: 0.0,
            total_runs: 0,
            successful_runs: 0,
            average_duration: 0.0,
        }
    }

    pub fn add_result(&mut self, result: TestResult) {
        self.total_runs += 1;
        if result.passed {
            self.successful_runs += 1;
        }
        
        if result.score > self.best_score {
            self.best_score = result.score;
            self.best_config = Some(result.config.clone());
        }
        
        self.results.push(result);
        
        // Update average duration
        let total_duration: f64 = self.results.iter().map(|r| r.diagnostics.duration_seconds).sum();
        self.average_duration = total_duration / self.results.len() as f64;
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Test Suite: {}\n\
             Total Runs: {}\n\
             Successful Runs: {}\n\
             Success Rate: {:.1}%\n\
             Best Score: {:.3}\n\
             Average Duration: {:.2}s\n\
             Best Config:\n{}",
            self.name,
            self.total_runs,
            self.successful_runs,
            (self.successful_runs as f64 / self.total_runs as f64) * 100.0,
            self.best_score,
            self.average_duration,
            if let Some(ref config) = self.best_config {
                format!("  Initial Agents: {}\n  Initial Resources: {}\n  Max Agents: {}\n  Max Resources: {}\n  Resource Spawn Rate: {:.2}",
                    config.initial_agents, config.initial_resources, config.max_agents, config.max_resources, config.resource_spawn_rate)
            } else {
                "None".to_string()
            }
        )
    }
}

pub struct TestHarness {
    suites: HashMap<String, TestSuite>,
    current_suite: String,
}

impl TestHarness {
    pub fn new() -> Self {
        Self {
            suites: HashMap::new(),
            current_suite: "default".to_string(),
        }
    }

    pub fn create_suite(&mut self, name: String) {
        self.suites.insert(name.clone(), TestSuite::new(name.clone()));
        self.current_suite = name;
    }

    pub fn run_single_test(&mut self, config: HeadlessSimulationConfig) -> TestResult {
        println!("Running test with config: {} agents, {} resources", 
                 config.initial_agents, config.initial_resources);
        
        let start_time = Instant::now();
        let mut simulation = HeadlessSimulation::new(config.clone());
        let diagnostics = simulation.run();
        let run_time = start_time.elapsed();
        
        println!("Test completed in {:.2}s", run_time.as_secs_f64());
        
        let score = self.calculate_score(&diagnostics);
        let passed = self.evaluate_test(&diagnostics);
        let notes = self.generate_notes(&diagnostics);
        
        let result = TestResult {
            config,
            diagnostics,
            score,
            passed,
            notes,
        };
        
        if let Some(suite) = self.suites.get_mut(&self.current_suite) {
            suite.add_result(result.clone());
        }
        
        result
    }

    pub fn run_parameter_sweep(&mut self, base_config: HeadlessSimulationConfig) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        // Test different initial population sizes
        let agent_counts = vec![100, 200, 300, 400, 500, 600, 700, 800];
        let resource_counts = vec![200, 300, 400, 500, 600, 700, 800, 900];
        
        for &agents in &agent_counts {
            for &resources in &resource_counts {
                let mut config = base_config.clone();
                config.initial_agents = agents;
                config.initial_resources = resources;
                
                let result = self.run_single_test(config);
                results.push(result.clone());
                
                // Early termination if we find a good configuration
                if result.score > 0.8 {
                    println!("Found high-scoring configuration! Score: {:.3}", result.score);
                }
            }
        }
        
        results
    }

    pub fn run_optimization_iteration(&mut self, base_config: HeadlessSimulationConfig, iterations: usize) -> HeadlessSimulationConfig {
        let mut best_config = base_config.clone();
        let mut best_score = 0.0;
        
        for i in 0..iterations {
            println!("Optimization iteration {}/{}", i + 1, iterations);
            
            // Generate variations of the best config
            let variations = self.generate_config_variations(&best_config, 5);
            
            for config in variations {
                let result = self.run_single_test(config.clone());
                
                if result.score > best_score {
                    best_score = result.score;
                    best_config = config;
                    println!("New best score: {:.3}", best_score);
                }
            }
        }
        
        best_config
    }

    fn generate_config_variations(&self, base: &HeadlessSimulationConfig, count: usize) -> Vec<HeadlessSimulationConfig> {
        let mut variations = Vec::new();
        let mut rng = rand::thread_rng();
        
        for _ in 0..count {
            let mut config = base.clone();
            
            // Vary initial agents by ±20%
            let agent_variation = (base.initial_agents as f64 * 0.2) as usize;
            let agent_change = rng.gen_range(0..=agent_variation * 2) as i32 - agent_variation as i32;
            config.initial_agents = (base.initial_agents as i32 + agent_change).max(50) as usize;
            
            // Vary initial resources by ±20%
            let resource_variation = (base.initial_resources as f64 * 0.2) as usize;
            let resource_change = rng.gen_range(0..=resource_variation * 2) as i32 - resource_variation as i32;
            config.initial_resources = (base.initial_resources as i32 + resource_change).max(100) as usize;
            
            // Vary resource spawn rate by ±50%
            config.resource_spawn_rate = base.resource_spawn_rate * rng.gen_range(0.5..=1.5);
            
            // Vary stability threshold
            config.stability_threshold = base.stability_threshold * rng.gen_range(0.8..=1.2);
            
            variations.push(config);
        }
        
        variations
    }

    fn calculate_score(&self, diagnostics: &SimulationDiagnostics) -> f64 {
        let mut score = 0.0;
        
        // Base score for completing the target duration
        if diagnostics.duration_seconds >= diagnostics.config.target_duration_minutes * 60.0 * 0.8 {
            score += 0.3;
        }
        
        // Stability score
        score += diagnostics.stability_score * 0.3;
        
        // Dynamic score
        if diagnostics.is_dynamic {
            score += 0.2;
        }
        
        // Population health score
        let final_agents = diagnostics.final_stats.agent_count;
        let target_agents = diagnostics.config.initial_agents;
        let agent_ratio = final_agents as f64 / target_agents as f64;
        
        if agent_ratio >= 0.5 && agent_ratio <= 2.0 {
            score += 0.2;
        }
        
        // Penalty for extinction or explosion
        if diagnostics.extinction_occurred {
            score -= 0.5;
        }
        if diagnostics.population_explosion {
            score -= 0.3;
        }
        
        // Bonus for good reproduction and evolution
        if diagnostics.average_generations > 1.0 {
            score += 0.1;
        }
        
        score.max(0.0).min(1.0)
    }

    fn evaluate_test(&self, diagnostics: &SimulationDiagnostics) -> bool {
        // Test passes if:
        // 1. No extinction occurred
        // 2. No population explosion
        // 3. Final population is reasonable
        // 4. Simulation ran for most of target duration
        
        let duration_ratio = diagnostics.duration_seconds / (diagnostics.config.target_duration_minutes * 60.0);
        let final_agents = diagnostics.final_stats.agent_count;
        let target_agents = diagnostics.config.initial_agents;
        let agent_ratio = final_agents as f64 / target_agents as f64;
        
        !diagnostics.extinction_occurred &&
        !diagnostics.population_explosion &&
        agent_ratio >= 0.3 && agent_ratio <= 3.0 &&
        duration_ratio >= 0.8
    }

    fn generate_notes(&self, diagnostics: &SimulationDiagnostics) -> String {
        let mut notes = Vec::new();
        
        if diagnostics.extinction_occurred {
            notes.push("Extinction occurred".to_string());
        }
        if diagnostics.population_explosion {
            notes.push("Population explosion".to_string());
        }
        if diagnostics.is_stable {
            notes.push("Stable population".to_string());
        }
        if diagnostics.is_dynamic {
            notes.push("Dynamic population".to_string());
        }
        
        let final_agents = diagnostics.final_stats.agent_count;
        let target_agents = diagnostics.config.initial_agents;
        let agent_ratio = final_agents as f64 / target_agents as f64;
        
        if agent_ratio < 0.5 {
            notes.push("Population declined significantly".to_string());
        } else if agent_ratio > 2.0 {
            notes.push("Population grew significantly".to_string());
        }
        
        if diagnostics.average_generations > 2.0 {
            notes.push("Good evolutionary progress".to_string());
        }
        
        if notes.is_empty() {
            notes.push("Balanced simulation".to_string());
        }
        
        notes.join(", ")
    }

    pub fn print_summary(&self) {
        for (name, suite) in &self.suites {
            println!("\n{}", suite.get_summary());
        }
    }

    pub fn get_best_config(&self) -> Option<HeadlessSimulationConfig> {
        self.suites.values()
            .filter_map(|suite| suite.best_config.clone())
            .max_by(|a, b| {
                let suite_a = self.suites.values().find(|s| s.best_config.as_ref() == Some(a)).unwrap();
                let suite_b = self.suites.values().find(|s| s.best_config.as_ref() == Some(b)).unwrap();
                suite_a.best_score.partial_cmp(&suite_b.best_score).unwrap()
            })
    }
} 