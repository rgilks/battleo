use crate::simulation_core::{SimulationConfig, SimulationStats, UnifiedSimulation};
use rand::prelude::*;
use serde::Serialize;
use std::time::{Duration, Instant};

#[derive(Clone, Serialize)]
pub struct HeadlessSimulationConfig {
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
    pub speed_multiplier: f64, // For high-speed evaluation
}

impl Default for HeadlessSimulationConfig {
    fn default() -> Self {
        Self {
            width: 1000.0,
            height: 800.0,
            max_agents: 5000,
            max_resources: 2000,
            initial_agents: 500,
            initial_resources: 500,
            resource_spawn_rate: 0.2,
            target_duration_minutes: 5.0,
            stability_threshold: 0.1,
            min_agent_count: 10,
            max_agent_count: 3000,
            use_ecs: true,
            speed_multiplier: 10.0, // 10x faster than real-time
        }
    }
}

impl From<HeadlessSimulationConfig> for SimulationConfig {
    fn from(config: HeadlessSimulationConfig) -> Self {
        SimulationConfig {
            width: config.width,
            height: config.height,
            max_agents: config.max_agents,
            max_resources: config.max_resources,
            initial_agents: config.initial_agents,
            initial_resources: config.initial_resources,
            resource_spawn_rate: config.resource_spawn_rate,
            target_duration_minutes: config.target_duration_minutes,
            stability_threshold: config.stability_threshold,
            min_agent_count: config.min_agent_count,
            max_agent_count: config.max_agent_count,
            use_ecs: config.use_ecs,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct SimulationDiagnostics {
    pub config: HeadlessSimulationConfig,
    pub duration_seconds: f64,
    pub total_steps: usize,
    pub steps_per_second: f64,
    pub final_stats: SimulationStats,
    pub stability_score: f64,
    pub is_stable: bool,
    pub is_dynamic: bool,
    pub agent_count_history: Vec<usize>,
    pub resource_count_history: Vec<usize>,
    pub energy_history: Vec<f64>,
    pub fitness_history: Vec<f64>,
    pub extinction_occurred: bool,
    pub population_explosion: bool,
    pub average_generations: f64,
    pub total_reproductions: usize,
    pub total_deaths: usize,
    pub simulation_quality_score: f64,
}

pub struct HeadlessSimulationV2 {
    simulation: UnifiedSimulation,
    config: HeadlessSimulationConfig,
    diagnostics: SimulationDiagnostics,
    step_count: usize,
    start_time: Instant,
    history_interval: usize,
    last_stats_time: f64,
}

impl HeadlessSimulationV2 {
    pub fn new(config: HeadlessSimulationConfig) -> Self {
        let simulation_config: SimulationConfig = config.clone().into();
        let simulation = UnifiedSimulation::new(simulation_config);

        let diagnostics = SimulationDiagnostics {
            config: config.clone(),
            duration_seconds: 0.0,
            total_steps: 0,
            steps_per_second: 0.0,
            final_stats: simulation.get_stats(),
            stability_score: 0.0,
            is_stable: false,
            is_dynamic: false,
            agent_count_history: Vec::new(),
            resource_count_history: Vec::new(),
            energy_history: Vec::new(),
            fitness_history: Vec::new(),
            extinction_occurred: false,
            population_explosion: false,
            average_generations: 0.0,
            total_reproductions: 0,
            total_deaths: 0,
            simulation_quality_score: 0.0,
        };

        // Calculate history interval based on speed multiplier
        // Record history every 60 steps (1 second at 60 FPS) adjusted for speed
        let history_interval = (60.0 / config.speed_multiplier).max(1.0) as usize;

        Self {
            simulation,
            config,
            diagnostics,
            step_count: 0,
            start_time: Instant::now(),
            history_interval,
            last_stats_time: 0.0,
        }
    }

    pub fn run(&mut self) -> SimulationDiagnostics {
        let target_steps = (self.config.target_duration_minutes * 60.0 * 60.0 * self.config.speed_multiplier) as usize;

        println!("Starting headless simulation with {}x speed multiplier", self.config.speed_multiplier);
        println!("Target duration: {:.1} minutes", self.config.target_duration_minutes);
        println!("Target steps: {}", target_steps);
        println!("Using {} engine", if self.config.use_ecs { "ECS" } else { "Legacy" });

        while self.step_count < target_steps {
            self.step();

            // Check for early termination conditions
            if self.should_terminate_early() {
                println!("Early termination at step {}", self.step_count);
                break;
            }

            // Progress reporting
            if self.step_count % 10000 == 0 {
                let progress = (self.step_count as f64 / target_steps as f64) * 100.0;
                let elapsed = self.start_time.elapsed().as_secs_f64();
                let steps_per_sec = self.step_count as f64 / elapsed;
                println!("Progress: {:.1}% ({}/{} steps, {:.0} steps/sec)", 
                    progress, self.step_count, target_steps, steps_per_sec);
            }
        }

        self.finalize_diagnostics();
        self.diagnostics.clone()
    }

    fn step(&mut self) {
        self.simulation.update();
        self.step_count += 1;

        // Record history periodically
        if self.step_count % self.history_interval == 0 {
            let stats = self.simulation.get_stats();
            self.diagnostics.agent_count_history.push(stats.agent_count);
            self.diagnostics.resource_count_history.push(stats.resource_count);
            self.diagnostics.energy_history.push(stats.total_energy);
            self.diagnostics.fitness_history.push(stats.average_fitness);
        }
    }

    fn should_terminate_early(&self) -> bool {
        let stats = self.simulation.get_stats();

        // Check for extinction
        if stats.agent_count == 0 {
            return true;
        }

        // Check for population explosion
        if stats.agent_count > self.config.max_agent_count {
            return true;
        }

        // Check for population collapse
        if stats.agent_count < self.config.min_agent_count {
            return true;
        }

        false
    }

    fn finalize_diagnostics(&mut self) {
        let duration = self.start_time.elapsed();
        self.diagnostics.duration_seconds = duration.as_secs_f64();
        self.diagnostics.total_steps = self.step_count;
        self.diagnostics.steps_per_second = self.step_count as f64 / duration.as_secs_f64();

        let final_stats = self.simulation.get_stats();
        self.diagnostics.final_stats = final_stats.clone();

        // Calculate stability score
        self.diagnostics.stability_score = self.calculate_stability_score();
        self.diagnostics.is_stable = self.diagnostics.stability_score > self.config.stability_threshold;

        // Calculate dynamic score
        self.diagnostics.is_dynamic = self.calculate_dynamic_score();

        // Check for extinction/explosion
        self.diagnostics.extinction_occurred = final_stats.agent_count == 0;
        self.diagnostics.population_explosion = final_stats.agent_count > self.config.max_agent_count;

        // Calculate average generations and reproduction stats
        let total_generations: u32 = self.simulation.get_agents().iter().map(|a| a.generation).sum();
        self.diagnostics.average_generations = if final_stats.agent_count > 0 {
            total_generations as f64 / final_stats.agent_count as f64
        } else {
            0.0
        };

        // Count total reproductions and deaths (approximate)
        let initial_agents = self.config.initial_agents;
        let current_agents = final_stats.agent_count;
        let total_born = final_stats.agent_count.saturating_sub(initial_agents);
        self.diagnostics.total_reproductions = total_born;
        self.diagnostics.total_deaths = initial_agents.saturating_sub(current_agents) + total_born;

        // Calculate overall simulation quality score
        self.diagnostics.simulation_quality_score = self.calculate_quality_score();
    }

    fn calculate_stability_score(&self) -> f64 {
        if self.diagnostics.agent_count_history.len() < 10 {
            return 0.0;
        }

        // Calculate coefficient of variation for agent count
        let mean = self.diagnostics.agent_count_history.iter().sum::<usize>() as f64
            / self.diagnostics.agent_count_history.len() as f64;

        if mean == 0.0 {
            return 0.0;
        }

        let variance = self
            .diagnostics
            .agent_count_history
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / self.diagnostics.agent_count_history.len() as f64;

        let std_dev = variance.sqrt();
        let cv = std_dev / mean;

        // Convert to stability score (lower CV = higher stability)
        1.0 / (1.0 + cv)
    }

    fn calculate_dynamic_score(&self) -> bool {
        if self.diagnostics.agent_count_history.len() < 10 {
            return false;
        }

        // Check if there's meaningful variation in agent count
        let min_agents = *self
            .diagnostics
            .agent_count_history
            .iter()
            .min()
            .unwrap_or(&0);
        let max_agents = *self
            .diagnostics
            .agent_count_history
            .iter()
            .max()
            .unwrap_or(&0);
        let range = max_agents.saturating_sub(min_agents);

        // Consider dynamic if there's at least 10% variation in population
        let mean_agents = self.diagnostics.agent_count_history.iter().sum::<usize>()
            / self.diagnostics.agent_count_history.len();
        let variation_threshold = (mean_agents as f64 * 0.1) as usize;

        range > variation_threshold
    }

    fn calculate_quality_score(&self) -> f64 {
        let mut score = 0.0;

        // Duration completion (20%)
        let duration_ratio = self.diagnostics.duration_seconds / (self.config.target_duration_minutes * 60.0);
        score += duration_ratio * 0.2;

        // Population health (25%)
        let final_agents = self.diagnostics.final_stats.agent_count;
        let target_agents = self.config.initial_agents;
        let agent_ratio = final_agents as f64 / target_agents as f64;
        if agent_ratio >= 0.5 && agent_ratio <= 2.0 {
            score += 0.25;
        } else if agent_ratio >= 0.3 && agent_ratio <= 3.0 {
            score += 0.15;
        }

        // Stability (20%)
        score += self.diagnostics.stability_score * 0.2;

        // Evolution progress (15%)
        if self.diagnostics.average_generations > 1.0 {
            score += 0.15;
        } else if self.diagnostics.average_generations > 0.5 {
            score += 0.10;
        }

        // Dynamic behavior (10%)
        if self.diagnostics.is_dynamic {
            score += 0.10;
        }

        // Performance (10%)
        let target_steps_per_sec = 60.0 * self.config.speed_multiplier;
        let actual_steps_per_sec = self.diagnostics.steps_per_second;
        let performance_ratio = actual_steps_per_sec / target_steps_per_sec;
        score += performance_ratio.min(1.0) * 0.10;

        // Penalties
        if self.diagnostics.extinction_occurred {
            score -= 0.5;
        }
        if self.diagnostics.population_explosion {
            score -= 0.3;
        }

        score.max(0.0).min(1.0)
    }

    pub fn get_current_stats(&self) -> SimulationStats {
        self.simulation.get_stats()
    }

    pub fn get_diagnostics(&self) -> &SimulationDiagnostics {
        &self.diagnostics
    }

    pub fn print_summary(&self) {
        println!("\n=== Headless Simulation Summary ===");
        println!("Duration: {:.2}s", self.diagnostics.duration_seconds);
        println!("Total steps: {}", self.diagnostics.total_steps);
        println!("Steps per second: {:.1}", self.diagnostics.steps_per_second);
        println!("Speed multiplier: {}x", self.config.speed_multiplier);
        println!("Engine: {}", if self.config.use_ecs { "ECS" } else { "Legacy" });

        println!("\n=== Final Population ===");
        println!("Final agents: {}", self.diagnostics.final_stats.agent_count);
        println!("Final resources: {}", self.diagnostics.final_stats.resource_count);
        println!("Total energy: {:.1}", self.diagnostics.final_stats.total_energy);

        println!("\n=== Simulation Quality ===");
        println!("Stability score: {:.3}", self.diagnostics.stability_score);
        println!("Is stable: {}", self.diagnostics.is_stable);
        println!("Is dynamic: {}", self.diagnostics.is_dynamic);
        println!("Quality score: {:.3}", self.diagnostics.simulation_quality_score);
        println!("Extinction occurred: {}", self.diagnostics.extinction_occurred);
        println!("Population explosion: {}", self.diagnostics.population_explosion);
        println!("Average generations: {:.1}", self.diagnostics.average_generations);
        println!("Total reproductions: {}", self.diagnostics.total_reproductions);
        println!("Total deaths: {}", self.diagnostics.total_deaths);
    }
} 