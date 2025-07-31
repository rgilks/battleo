use crate::ecs::{EcsWorld, Genes, Position, Velocity, Energy, Age, AgentState, Resource, Size};
use rand::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
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

pub struct EcsSimulation {
    pub ecs_world: EcsWorld,
    pub width: f64,
    pub height: f64,
    pub time: f64,
    pub max_agents: usize,
    pub max_resources: usize,
}

impl EcsSimulation {
    pub fn is_rayon_available() -> bool {
        unsafe { THREAD_POOL_AVAILABLE && RAYON_INITIALIZED }
    }

    pub fn set_rayon_initialized(initialized: bool) {
        unsafe { RAYON_INITIALIZED = initialized; }
    }

    pub fn new(width: f64, height: f64) -> Self {
        let ecs_world = EcsWorld::new(width, height);
        
        Self {
            ecs_world,
            width,
            height,
            time: 0.0,
            max_agents: 10000,
            max_resources: 1500,
        }
    }

    pub fn update(&mut self) {
        let delta_time = 1.0 / 60.0;
        self.time += delta_time;

        // Update the ECS world
        self.ecs_world.update();
    }

    pub fn add_agent(&mut self, x: f64, y: f64) {
        if self.ecs_world.get_agent_count() < self.max_agents {
            self.ecs_world.add_agent(x, y);
        }
    }

    pub fn add_resource(&mut self, x: f64, y: f64) {
        if self.ecs_world.get_resource_count() < self.max_resources {
            self.ecs_world.add_resource(x, y);
        }
    }

    pub fn reset(&mut self) {
        self.ecs_world.reset();
        self.time = 0.0;
    }

    pub fn get_stats(&self) -> SimulationStats {
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

        // Get all agents from ECS world
        let agents = self.ecs_world.get_agents();

        // Calculate statistics
        let (
            total_energy,
            total_age,
            total_speed,
            total_size,
            total_aggression,
            total_sense_range,
            total_efficiency,
            total_kills,
            max_generation,
            total_fitness,
        ) = if Self::is_rayon_available() {
            // Parallel processing with wasm_bindgen_rayon
            agents
                .par_iter()
                .fold(
                    || (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0, 0.0),
                    |(
                        energy,
                        age,
                        speed,
                        size,
                        aggression,
                        sense_range,
                        efficiency,
                        kills,
                        gen,
                        fitness,
                    ),
                     (_, _, energy_comp, age_comp, state, genes, size_comp)| {
                        let agent_fitness = energy_comp.current
                            * genes.speed
                            * genes.size
                            * genes.aggression;
                        (
                            energy + energy_comp.current,
                            age + age_comp.value,
                            speed + genes.speed,
                            size + size_comp.value,
                            aggression + genes.aggression,
                            sense_range + genes.sense_range,
                            efficiency + genes.energy_efficiency,
                            kills + state.kills,
                            gen.max(state.generation),
                            fitness + agent_fitness,
                        )
                    },
                )
                .reduce(
                    || (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0, 0.0),
                    |(e1, a1, s1, sz1, ag1, sr1, ef1, k1, g1, f1),
                     (e2, a2, s2, sz2, ag2, sr2, ef2, k2, g2, f2)| {
                        (
                            e1 + e2,
                            a1 + a2,
                            s1 + s2,
                            sz1 + sz2,
                            ag1 + ag2,
                            sr1 + sr2,
                            ef1 + ef2,
                            k1 + k2,
                            g1.max(g2),
                            f1 + f2,
                        )
                    },
                )
        } else {
            // Sequential processing for WebAssembly compatibility
            agents.iter().fold(
                (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0, 0.0),
                |(
                    energy,
                    age,
                    speed,
                    size,
                    aggression,
                    sense_range,
                    efficiency,
                    kills,
                    gen,
                    fitness,
                ),
                 (_, _, energy_comp, age_comp, state, genes, size_comp)| {
                    let agent_fitness = energy_comp.current
                        * genes.speed
                        * genes.size
                        * genes.aggression;
                    (
                        energy + energy_comp.current,
                        age + age_comp.value,
                        speed + genes.speed,
                        size + size_comp.value,
                        aggression + genes.aggression,
                        sense_range + genes.sense_range,
                        efficiency + genes.energy_efficiency,
                        kills + state.kills,
                        gen.max(state.generation),
                        fitness + agent_fitness,
                    )
                },
            )
        };

        SimulationStats {
            agent_count,
            resource_count,
            total_energy,
            average_age: total_age / agent_count as f64,
            average_speed: total_speed / agent_count as f64,
            average_size: total_size / agent_count as f64,
            average_aggression: total_aggression / agent_count as f64,
            average_sense_range: total_sense_range / agent_count as f64,
            average_energy_efficiency: total_efficiency / agent_count as f64,
            max_generation,
            total_kills,
            average_fitness: total_fitness / agent_count as f64,
        }
    }

    // Compatibility methods for the old simulation interface
    pub fn agents(&self) -> Vec<LegacyAgent> {
        self.ecs_world.get_agents()
            .into_iter()
            .map(|(pos, vel, energy, age, state, genes, size)| {
                LegacyAgent {
                    x: pos.x,
                    y: pos.y,
                    dx: vel.dx,
                    dy: vel.dy,
                    energy: energy.current,
                    max_energy: energy.max,
                    age: age.value,
                    genes: LegacyGenes {
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
                        crate::ecs::AgentStateEnum::Seeking => LegacyAgentState::Seeking,
                        crate::ecs::AgentStateEnum::Hunting => LegacyAgentState::Hunting,
                        crate::ecs::AgentStateEnum::Feeding => LegacyAgentState::Feeding,
                        crate::ecs::AgentStateEnum::Reproducing => LegacyAgentState::Reproducing,
                        crate::ecs::AgentStateEnum::Fighting => LegacyAgentState::Fighting,
                        crate::ecs::AgentStateEnum::Fleeing => LegacyAgentState::Fleeing,
                    },
                    last_reproduction: state.last_reproduction,
                    kills: state.kills,
                    generation: state.generation,
                    death_fade: 0.0,
                    death_reason: None,
                    is_dying: false,
                    spawn_fade: 1.0,
                    spawn_position: None,
                }
            })
            .collect()
    }

    pub fn resources(&self) -> Vec<LegacyResource> {
        self.ecs_world.get_resources()
            .into_iter()
            .map(|(pos, resource, size)| {
                LegacyResource {
                    x: pos.x,
                    y: pos.y,
                    energy: resource.energy,
                    max_energy: resource.max_energy,
                    size: size.value,
                    growth_rate: resource.growth_rate,
                    regeneration_rate: resource.regeneration_rate,
                    age: resource.age,
                    target_energy: resource.target_energy,
                    is_spawning: resource.is_spawning,
                    spawn_fade: resource.spawn_fade,
                    is_depleting: resource.is_depleting,
                    deplete_fade: resource.deplete_fade,
                }
            })
            .collect()
    }
}

// Legacy types for compatibility with existing code
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyAgent {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub energy: f64,
    pub max_energy: f64,
    pub age: f64,
    pub genes: LegacyGenes,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub state: LegacyAgentState,
    pub last_reproduction: f64,
    pub kills: u32,
    pub generation: u32,
    pub death_fade: f64,
    pub death_reason: Option<LegacyDeathReason>,
    pub is_dying: bool,
    pub spawn_fade: f64,
    pub spawn_position: Option<(f64, f64)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LegacyAgentState {
    Seeking,
    Hunting,
    Feeding,
    Reproducing,
    Fighting,
    Fleeing,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LegacyDeathReason {
    Starvation,
    OldAge,
    KilledByPredator,
    Combat,
    NaturalCauses,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyGenes {
    pub speed: f64,
    pub sense_range: f64,
    pub size: f64,
    pub energy_efficiency: f64,
    pub reproduction_threshold: f64,
    pub mutation_rate: f64,
    pub aggression: f64,
    pub color_hue: f64,
    pub is_predator: f64,
    pub hunting_speed: f64,
    pub attack_power: f64,
    pub defense: f64,
    pub stealth: f64,
    pub pack_mentality: f64,
    pub territory_size: f64,
    pub metabolism: f64,
    pub intelligence: f64,
    pub stamina: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyResource {
    pub x: f64,
    pub y: f64,
    pub energy: f64,
    pub max_energy: f64,
    pub size: f64,
    pub growth_rate: f64,
    pub regeneration_rate: f64,
    pub age: f64,
    pub target_energy: f64,
    pub is_spawning: bool,
    pub spawn_fade: f64,
    pub is_depleting: bool,
    pub deplete_fade: f64,
}

impl LegacyAgent {
    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn id(&self) -> u64 {
        ((self.x * 1000.0) as u64) ^ ((self.y * 1000.0) as u64) ^ (self.generation as u64)
    }

    pub fn is_alive(&self) -> bool {
        self.energy > 0.0 && self.age < 200.0
    }

    pub fn is_predator(&self) -> bool {
        self.genes.is_predator > 0.5
    }

    pub fn is_prey(&self) -> bool {
        !self.is_predator()
    }
}

impl LegacyResource {
    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn is_available(&self) -> bool {
        self.energy > 5.0 && !self.is_depleting && self.spawn_fade > 0.5
    }
} 