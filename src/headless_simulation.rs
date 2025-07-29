use crate::simulation::{Simulation, SimulationStats};
use rand::prelude::*;
use serde::Serialize;
use std::time::{Duration, Instant};

#[derive(Clone, Serialize, PartialEq)]
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
}

pub struct HeadlessSimulation {
    simulation: Simulation,
    config: HeadlessSimulationConfig,
    diagnostics: SimulationDiagnostics,
    step_count: usize,
    start_time: Instant,
    history_interval: usize,
}

impl HeadlessSimulation {
    pub fn new(config: HeadlessSimulationConfig) -> Self {
        let mut simulation = Simulation::new(config.width, config.height);

        // Override simulation limits with config
        simulation.max_agents = config.max_agents;
        simulation.max_resources = config.max_resources;

        // Reset and spawn with config values
        simulation.reset();

        // Clear existing agents and resources to start fresh
        simulation.agents.clear();
        simulation.resources.clear();

        // Ensure we have the right number of initial agents/resources
        while simulation.agents.len() < config.initial_agents {
            simulation.add_agent(
                rand::random::<f64>() * config.width,
                rand::random::<f64>() * config.height,
            );
        }

        while simulation.resources.len() < config.initial_resources {
            simulation.add_resource(
                rand::random::<f64>() * config.width,
                rand::random::<f64>() * config.height,
            );
        }

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
        };

        Self {
            simulation,
            config,
            diagnostics,
            step_count: 0,
            start_time: Instant::now(),
            history_interval: 6, // Record history every 6 steps (0.5 seconds at 12 FPS) - 10x more frequent
        }
    }

    pub fn run(&mut self) -> SimulationDiagnostics {
        let target_steps = (self.config.target_duration_minutes * 60.0 * 12.0) as usize; // Changed from 120.0 to 12.0 (10x faster)

        while self.step_count < target_steps {
            self.step();

            // Check for early termination conditions
            if self.should_terminate_early() {
                break;
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
            self.diagnostics
                .resource_count_history
                .push(stats.resource_count);
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
        self.diagnostics.is_stable =
            self.diagnostics.stability_score > self.config.stability_threshold;

        // Calculate dynamic score
        self.diagnostics.is_dynamic = self.calculate_dynamic_score();

        // Check for extinction/explosion
        self.diagnostics.extinction_occurred = final_stats.agent_count == 0;
        self.diagnostics.population_explosion =
            final_stats.agent_count > self.config.max_agent_count;

        // Calculate average generations and reproduction stats
        let total_generations: u32 = self.simulation.agents.iter().map(|a| a.generation).sum();
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

    pub fn get_current_stats(&self) -> SimulationStats {
        self.simulation.get_stats()
    }

    pub fn get_diagnostics(&self) -> &SimulationDiagnostics {
        &self.diagnostics
    }
}
