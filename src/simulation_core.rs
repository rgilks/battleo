use crate::agent::{Agent, AgentState};
use crate::ecs::{
    Age, AgentStateEnum, EcsWorld, Energy, Genes as EcsGenes, Position, Resource as EcsResource,
    Size, Velocity,
};
use crate::genes::Genes;
use crate::resource::Resource;
use rand::prelude::*;
use rayon::prelude::*;
use serde::Serialize;
use std::sync::Arc;

static mut THREAD_POOL_AVAILABLE: bool = false;
static mut RAYON_INITIALIZED: bool = false;

#[derive(Clone, Serialize)]
pub struct SimulationStats {
    pub agent_count: usize,
    pub resource_count: usize,
    pub total_energy: f64,
    pub average_age: f64,
    pub average_speed: f64,
    pub average_size: f64,
    pub average_aggression: f64,
    pub average_sense_range: f64,
    pub average_energy_efficiency: f64,
    pub max_generation: u32,
    pub total_kills: u32,
    pub average_fitness: f64,
}

#[derive(Clone, Serialize)]
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
    pub use_ecs: bool, // Whether to use ECS or legacy simulation
}

impl Default for SimulationConfig {
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
        }
    }
}

pub trait SimulationEngine {
    fn update(&mut self);
    fn add_agent(&mut self, x: f64, y: f64);
    fn add_resource(&mut self, x: f64, y: f64);
    fn reset(&mut self);
    fn get_stats(&self) -> SimulationStats;
    fn get_agents(&self) -> Vec<Agent>;
    fn get_resources(&self) -> Vec<Resource>;
    fn get_config(&self) -> &SimulationConfig;
}

pub struct EcsSimulationEngine {
    ecs_world: EcsWorld,
    config: SimulationConfig,
    time: f64,
}

impl EcsSimulationEngine {
    pub fn new(config: SimulationConfig) -> Self {
        let ecs_world = EcsWorld::new(config.width, config.height);

        Self {
            ecs_world,
            config,
            time: 0.0,
        }
    }

    pub fn is_rayon_available() -> bool {
        unsafe { THREAD_POOL_AVAILABLE && RAYON_INITIALIZED }
    }

    pub fn set_rayon_initialized(initialized: bool) {
        unsafe {
            RAYON_INITIALIZED = initialized;
        }
    }
}

impl SimulationEngine for EcsSimulationEngine {
    fn update(&mut self) {
        let delta_time = 1.0 / 60.0;
        self.time += delta_time;
        self.ecs_world.update();
    }

    fn add_agent(&mut self, x: f64, y: f64) {
        if self.ecs_world.get_agent_count() < self.config.max_agents {
            self.ecs_world.add_agent(x, y);
        }
    }

    fn add_resource(&mut self, x: f64, y: f64) {
        if self.ecs_world.get_resource_count() < self.config.max_resources {
            self.ecs_world.add_resource(x, y);
        }
    }

    fn reset(&mut self) {
        self.ecs_world.reset();
        self.time = 0.0;
    }

    fn get_stats(&self) -> SimulationStats {
        let agent_count = self.ecs_world.get_agent_count();
        let resource_count = self.ecs_world.get_resource_count();

        if agent_count == 0 {
            return SimulationStats {
                agent_count: 0,
                resource_count,
                total_energy: 0.0,
                average_age: 0.0,
                average_speed: 0.0,
                average_size: 0.0,
                average_aggression: 0.0,
                average_sense_range: 0.0,
                average_energy_efficiency: 0.0,
                max_generation: 0,
                total_kills: 0,
                average_fitness: 0.0,
            };
        }

        // Get all agents for statistics
        let agents = self.get_agents();

        let total_energy: f64 = agents.iter().map(|a| a.energy).sum();
        let average_age: f64 = agents.iter().map(|a| a.age).sum::<f64>() / agent_count as f64;
        let average_speed: f64 =
            agents.iter().map(|a| a.genes.speed).sum::<f64>() / agent_count as f64;
        let average_size: f64 =
            agents.iter().map(|a| a.genes.size).sum::<f64>() / agent_count as f64;
        let average_aggression: f64 =
            agents.iter().map(|a| a.genes.aggression).sum::<f64>() / agent_count as f64;
        let average_sense_range: f64 =
            agents.iter().map(|a| a.genes.sense_range).sum::<f64>() / agent_count as f64;
        let average_energy_efficiency: f64 = agents
            .iter()
            .map(|a| a.genes.energy_efficiency)
            .sum::<f64>()
            / agent_count as f64;
        let max_generation = agents.iter().map(|a| a.generation).max().unwrap_or(0);
        let total_kills: u32 = agents.iter().map(|a| a.kills).sum();
        let average_fitness: f64 =
            agents.iter().map(|a| a.energy / a.max_energy).sum::<f64>() / agent_count as f64;

        SimulationStats {
            agent_count,
            resource_count,
            total_energy,
            average_age,
            average_speed,
            average_size,
            average_aggression,
            average_sense_range,
            average_energy_efficiency,
            max_generation,
            total_kills,
            average_fitness,
        }
    }

    fn get_agents(&self) -> Vec<Agent> {
        // Convert ECS agents to legacy Agent format for compatibility
        self.ecs_world
            .get_agents()
            .into_iter()
            .map(|(pos, vel, energy, age, state, genes, _size)| Agent {
                x: pos.x,
                y: pos.y,
                dx: vel.dx,
                dy: vel.dy,
                energy: energy.current,
                max_energy: energy.max,
                age: age.value,
                genes: Genes {
                    speed: genes.speed,
                    sense_range: genes.sense_range,
                    size: genes.size,
                    energy_efficiency: genes.energy_efficiency,
                    reproduction_threshold: genes.reproduction_threshold,
                    mutation_rate: genes.mutation_rate,
                    aggression: genes.aggression,
                    color_hue: genes.color_hue,
                    is_predator: genes.is_predator,
                    hunting_speed: genes.hunting_speed,
                    attack_power: genes.attack_power,
                    defense: genes.defense,
                    stealth: genes.stealth,
                    pack_mentality: genes.pack_mentality,
                    territory_size: genes.territory_size,
                    metabolism: genes.metabolism,
                    intelligence: genes.intelligence,
                    stamina: genes.stamina,
                },
                target_x: state.target_x,
                target_y: state.target_y,
                state: match state.state {
                    AgentStateEnum::Seeking => AgentState::Seeking,
                    AgentStateEnum::Hunting => AgentState::Hunting,
                    AgentStateEnum::Feeding => AgentState::Feeding,
                    AgentStateEnum::Reproducing => AgentState::Reproducing,
                    AgentStateEnum::Fighting => AgentState::Fighting,
                    AgentStateEnum::Fleeing => AgentState::Fleeing,
                },
                last_reproduction: state.last_reproduction,
                kills: state.kills,
                generation: state.generation,
                death_fade: 0.0,
                death_reason: None,
                is_dying: false,
                spawn_fade: 0.0,
                spawn_position: None,
            })
            .collect()
    }

    fn get_resources(&self) -> Vec<Resource> {
        // Convert ECS resources to legacy Resource format for compatibility
        self.ecs_world
            .get_resources()
            .into_iter()
            .map(|(pos, ecs_resource, size)| Resource {
                x: pos.x,
                y: pos.y,
                energy: ecs_resource.energy,
                max_energy: ecs_resource.max_energy,
                size: size.value,
                growth_rate: ecs_resource.growth_rate,
                regeneration_rate: ecs_resource.regeneration_rate,
                age: ecs_resource.age,
                target_energy: ecs_resource.target_energy,
                is_spawning: ecs_resource.is_spawning,
                spawn_fade: ecs_resource.spawn_fade,
                is_depleting: ecs_resource.is_depleting,
                deplete_fade: ecs_resource.deplete_fade,
            })
            .collect()
    }

    fn get_config(&self) -> &SimulationConfig {
        &self.config
    }
}

pub struct LegacySimulationEngine {
    agents: Vec<Agent>,
    resources: Vec<Resource>,
    config: SimulationConfig,
    time: f64,
    resource_spawn_timer: f64,
    grid_cell_size: f64,
    grid_width: usize,
    grid_height: usize,
    spatial_grid: Vec<Vec<Vec<usize>>>,
}

impl LegacySimulationEngine {
    pub fn new(config: SimulationConfig) -> Self {
        let grid_cell_size = 50.0;
        let grid_width = (config.width / grid_cell_size).ceil() as usize;
        let grid_height = (config.height / grid_cell_size).ceil() as usize;
        let spatial_grid = vec![vec![Vec::new(); grid_height]; grid_width];

        let mut engine = Self {
            agents: Vec::new(),
            resources: Vec::new(),
            config,
            time: 0.0,
            resource_spawn_timer: 0.0,
            grid_cell_size,
            grid_width,
            grid_height,
            spatial_grid,
        };

        engine.spawn_initial_population();
        engine
    }

    fn get_grid_position(&self, x: f64, y: f64) -> (usize, usize) {
        let grid_x = (x / self.grid_cell_size).floor() as usize;
        let grid_y = (y / self.grid_cell_size).floor() as usize;
        (
            grid_x.min(self.grid_width - 1),
            grid_y.min(self.grid_height - 1),
        )
    }

    fn get_nearby_agents(&self, x: f64, y: f64, radius: f64) -> Vec<usize> {
        let mut nearby = Vec::new();
        let (center_x, center_y) = self.get_grid_position(x, y);

        // Check current cell and adjacent cells
        for dx in -1..=1 {
            for dy in -1..=1 {
                let check_x = center_x as i32 + dx;
                let check_y = center_y as i32 + dy;

                if check_x >= 0
                    && check_x < self.grid_width as i32
                    && check_y >= 0
                    && check_y < self.grid_height as i32
                {
                    let cell_agents = &self.spatial_grid[check_x as usize][check_y as usize];
                    for &agent_idx in cell_agents {
                        if agent_idx < self.agents.len() {
                            let agent = &self.agents[agent_idx];
                            let distance = ((agent.x - x).powi(2) + (agent.y - y).powi(2)).sqrt();
                            if distance <= radius {
                                nearby.push(agent_idx);
                            }
                        }
                    }
                }
            }
        }
        nearby
    }

    fn update_spatial_grid(&mut self) {
        // Clear grid
        for row in &mut self.spatial_grid {
            for cell in row {
                cell.clear();
            }
        }

        // Rebuild grid
        for (i, agent) in self.agents.iter().enumerate() {
            let (grid_x, grid_y) = self.get_grid_position(agent.x, agent.y);
            if grid_x < self.grid_width && grid_y < self.grid_height {
                self.spatial_grid[grid_x][grid_y].push(i);
            }
        }
    }

    fn spawn_initial_population(&mut self) {
        let mut rng = rand::thread_rng();

        // Spawn initial agents
        for _ in 0..self.config.initial_agents {
            let x = rng.gen_range(0.0..self.config.width);
            let y = rng.gen_range(0.0..self.config.height);
            self.add_agent(x, y);
        }

        // Spawn initial resources
        for _ in 0..self.config.initial_resources {
            let x = rng.gen_range(0.0..self.config.width);
            let y = rng.gen_range(0.0..self.config.height);
            self.add_resource(x, y);
        }

        self.update_spatial_grid();
    }

    fn spawn_resource(&mut self) {
        if self.resources.len() < self.config.max_resources {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(0.0..self.config.width);
            let y = rng.gen_range(0.0..self.config.height);
            self.add_resource(x, y);
        }
    }

    fn cleanup_dead_agents(&mut self) {
        self.agents.retain(|agent| agent.energy > 0.0);
    }

    fn cleanup_depleted_resources(&mut self) {
        self.resources.retain(|resource| resource.energy > 0.0);
    }
}

impl SimulationEngine for LegacySimulationEngine {
    fn update(&mut self) {
        let delta_time = 1.0 / 60.0;
        self.time += delta_time;
        self.resource_spawn_timer += delta_time;

        // Spawn resources periodically
        if self.resource_spawn_timer >= 1.0 / self.config.resource_spawn_rate {
            self.spawn_resource();
            self.resource_spawn_timer = 0.0;
        }

        // Update agents and resources (simplified for now)
        for agent in &mut self.agents {
            agent.age += delta_time;
            agent.energy -= delta_time * 0.1; // Basic energy consumption
        }

        for resource in &mut self.resources {
            resource.age += delta_time;
            if resource.energy < resource.max_energy {
                resource.energy += delta_time * resource.regeneration_rate;
            }
        }

        self.cleanup_dead_agents();
        self.cleanup_depleted_resources();
        self.update_spatial_grid();
    }

    fn add_agent(&mut self, x: f64, y: f64) {
        if self.agents.len() < self.config.max_agents {
            let genes = Genes::new();
            let agent = Agent::new(x, y, genes, 1);
            self.agents.push(agent);
        }
    }

    fn add_resource(&mut self, x: f64, y: f64) {
        if self.resources.len() < self.config.max_resources {
            let resource = Resource::new(x, y);
            self.resources.push(resource);
        }
    }

    fn reset(&mut self) {
        self.agents.clear();
        self.resources.clear();
        self.time = 0.0;
        self.resource_spawn_timer = 0.0;
        self.spawn_initial_population();
    }

    fn get_stats(&self) -> SimulationStats {
        let agent_count = self.agents.len();
        let resource_count = self.resources.len();

        if agent_count == 0 {
            return SimulationStats {
                agent_count: 0,
                resource_count,
                total_energy: 0.0,
                average_age: 0.0,
                average_speed: 0.0,
                average_size: 0.0,
                average_aggression: 0.0,
                average_sense_range: 0.0,
                average_energy_efficiency: 0.0,
                max_generation: 0,
                total_kills: 0,
                average_fitness: 0.0,
            };
        }

        let total_energy: f64 = self.agents.iter().map(|a| a.energy).sum();
        let average_age: f64 = self.agents.iter().map(|a| a.age).sum::<f64>() / agent_count as f64;
        let average_speed: f64 =
            self.agents.iter().map(|a| a.genes.speed).sum::<f64>() / agent_count as f64;
        let average_size: f64 =
            self.agents.iter().map(|a| a.genes.size).sum::<f64>() / agent_count as f64;
        let average_aggression: f64 =
            self.agents.iter().map(|a| a.genes.aggression).sum::<f64>() / agent_count as f64;
        let average_sense_range: f64 =
            self.agents.iter().map(|a| a.genes.sense_range).sum::<f64>() / agent_count as f64;
        let average_energy_efficiency: f64 = self
            .agents
            .iter()
            .map(|a| a.genes.energy_efficiency)
            .sum::<f64>()
            / agent_count as f64;
        let max_generation = self.agents.iter().map(|a| a.generation).max().unwrap_or(0);
        let total_kills: u32 = self.agents.iter().map(|a| a.kills).sum();
        let average_fitness: f64 = self
            .agents
            .iter()
            .map(|a| a.energy / a.max_energy)
            .sum::<f64>()
            / agent_count as f64;

        SimulationStats {
            agent_count,
            resource_count,
            total_energy,
            average_age,
            average_speed,
            average_size,
            average_aggression,
            average_sense_range,
            average_energy_efficiency,
            max_generation,
            total_kills,
            average_fitness,
        }
    }

    fn get_agents(&self) -> Vec<Agent> {
        self.agents.clone()
    }

    fn get_resources(&self) -> Vec<Resource> {
        self.resources.clone()
    }

    fn get_config(&self) -> &SimulationConfig {
        &self.config
    }
}

pub struct UnifiedSimulation {
    engine: Box<dyn SimulationEngine>,
    config: SimulationConfig,
}

impl UnifiedSimulation {
    pub fn new(config: SimulationConfig) -> Self {
        let engine: Box<dyn SimulationEngine> = if config.use_ecs {
            Box::new(EcsSimulationEngine::new(config.clone()))
        } else {
            Box::new(LegacySimulationEngine::new(config.clone()))
        };

        Self { engine, config }
    }

    pub fn update(&mut self) {
        self.engine.update();
    }

    pub fn add_agent(&mut self, x: f64, y: f64) {
        self.engine.add_agent(x, y);
    }

    pub fn add_resource(&mut self, x: f64, y: f64) {
        self.engine.add_resource(x, y);
    }

    pub fn reset(&mut self) {
        self.engine.reset();
    }

    pub fn get_stats(&self) -> SimulationStats {
        self.engine.get_stats()
    }

    pub fn get_agents(&self) -> Vec<Agent> {
        self.engine.get_agents()
    }

    pub fn get_resources(&self) -> Vec<Resource> {
        self.engine.get_resources()
    }

    pub fn get_config(&self) -> &SimulationConfig {
        &self.config
    }

    pub fn is_rayon_available() -> bool {
        EcsSimulationEngine::is_rayon_available()
    }

    pub fn set_rayon_initialized(initialized: bool) {
        EcsSimulationEngine::set_rayon_initialized(initialized);
    }
}
