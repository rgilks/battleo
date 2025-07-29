use crate::agent::Agent;
use crate::genes::Genes;
use crate::resource::Resource;
use rand::prelude::*;
use rand_distr::{Normal, Uniform};
use rayon::prelude::*;
use serde::Serialize;
use std::sync::{Arc, Mutex};

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

pub struct Simulation {
    pub agents: Vec<Agent>,
    pub resources: Vec<Resource>,
    pub width: f64,
    pub height: f64,
    pub time: f64,
    pub resource_spawn_timer: f64,
    pub max_agents: usize,
    pub max_resources: usize,
}

impl Simulation {
    pub fn new(width: f64, height: f64) -> Self {
        let mut simulation = Self {
            agents: Vec::new(),
            resources: Vec::new(),
            width,
            height,
            time: 0.0,
            resource_spawn_timer: 0.0,
            max_agents: 5000,    // Increased from 1000
            max_resources: 2000, // Increased from 1000 - more food
        };

        // Initialize with some agents and resources
        simulation.spawn_initial_population();

        simulation
    }

    fn spawn_initial_population(&mut self) {
        let mut rng = thread_rng();

        // Spawn initial agents - much smaller for web interface
        let initial_agents = (self.max_agents as f64 * 0.05) as usize; // Reduced from 0.1 to 0.05 (5% of max)
        for _ in 0..initial_agents {
            let x = rng.gen_range(0.0..self.width);
            let y = rng.gen_range(0.0..self.height);
            let genes = Genes::new();
            let agent = Agent::new(x, y, genes, 0);
            self.agents.push(agent);
        }

        // Spawn initial resources - much smaller for web interface
        let initial_resources = (self.max_resources as f64 * 0.15) as usize; // Reduced from 0.3 to 0.15 (15% of max)
        for _ in 0..initial_resources {
            let x = rng.gen_range(0.0..self.width);
            let y = rng.gen_range(0.0..self.height);
            let resource = Resource::new(x, y);
            self.resources.push(resource);
        }
    }

    pub fn update(&mut self) {
        let delta_time = 1.0 / 12.0; // Changed from 1.0/120.0 to 1.0/12.0 (10x faster)
        self.time += delta_time;

        // Update resources in parallel
        self.update_resources_parallel(delta_time);

        // Spawn new resources more frequently
        self.resource_spawn_timer += delta_time;
        if self.resource_spawn_timer > 0.02 && self.resources.len() < self.max_resources {
            // Much more frequent spawning (10x faster)
            self.spawn_resource();
            self.resource_spawn_timer = 0.0;
        }

        // Update agents with parallel processing
        self.update_agents_parallel(delta_time);

        // Handle reproduction in parallel
        self.handle_reproduction_parallel();

        // Clean up dead agents
        self.cleanup_dead_agents();

        // Add some complex calculations to utilize CPU cores
        self.perform_complex_calculations();
    }

    fn update_resources_parallel(&mut self, delta_time: f64) {
        // Use Rayon to update resources in parallel
        self.resources.par_iter_mut().for_each(|resource| {
            resource.update(delta_time);
        });
    }

    fn update_agents_parallel(&mut self, delta_time: f64) {
        // Create shared references for parallel processing
        let resources = Arc::new(self.resources.clone());

        // Process agents in parallel
        let agent_updates: Vec<_> = self
            .agents
            .par_iter_mut()
            .enumerate()
            .map(|(i, agent)| {
                let consumed = agent.update(delta_time, &resources, &[], self.width, self.height);
                (i, consumed)
            })
            .collect();

        // Collect consumed resources
        let mut consumed_indices = Vec::new();
        for (_, consumed) in agent_updates {
            if let Some(resource_index) = consumed {
                consumed_indices.push(resource_index);
            }
        }

        // Remove consumed resources (in reverse order to maintain indices)
        consumed_indices.sort_unstable();
        consumed_indices.reverse();
        for &index in &consumed_indices {
            if index < self.resources.len() {
                self.resources.remove(index);
            }
        }
    }

    fn handle_reproduction_parallel(&mut self) {
        // Use Rayon to handle reproduction in parallel
        let mut new_agents = Vec::new();

        // Much more restrictive population control
        let safe_population_threshold = (self.max_agents as f64 * 0.6) as usize; // Reduced from 0.8

        if self.agents.len() >= safe_population_threshold {
            return; // Don't reproduce if population is too high
        }

        // Only allow reproduction if population is growing slowly
        let population_growth_rate = self.agents.len() as f64 / self.max_agents as f64;
        if population_growth_rate > 0.4 {
            return; // Don't reproduce if population is already substantial
        }

        // Process reproduction in parallel
        let reproduction_results: Vec<_> = self
            .agents
            .par_iter()
            .filter_map(|agent| {
                if agent.can_reproduce() {
                    // Find a suitable mate nearby
                    let potential_mates: Vec<_> = self
                        .agents
                        .iter()
                        .filter(|other| {
                            other.id() != agent.id()
                                && agent.distance_to(other.x, other.y) < 20.0
                                && other.energy > 50.0 // Increased from 30.0
                                && other.can_reproduce()
                        })
                        .collect();

                    if let Some(mate) = potential_mates.choose(&mut thread_rng()) {
                        // Additional check: only reproduce if we won't exceed max agents
                        if self.agents.len() + new_agents.len() < self.max_agents {
                            // Random chance to reproduce (50% chance)
                            if thread_rng().gen::<f64>() < 0.5 {
                                Some(agent.create_offspring(mate))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Add new agents
        for new_agent in reproduction_results {
            if self.agents.len() + new_agents.len() < self.max_agents {
                new_agents.push(new_agent);
            }
        }

        self.agents.extend(new_agents);
    }

    fn perform_complex_calculations(&mut self) {
        // Much simplified calculations to avoid instability
        let num_agents = self.agents.len();
        let num_resources = self.resources.len();

        // Only do complex calculations if we have enough agents
        if num_agents > 10 && num_resources > 10 {
            // Simplified agent-to-agent interaction calculations
            let agent_positions: Vec<_> = self
                .agents
                .par_iter()
                .map(|agent| (agent.x, agent.y))
                .collect();

            // Calculate simplified interaction matrices in parallel
            let interaction_matrix: Vec<_> = agent_positions
                .par_iter()
                .enumerate()
                .map(|(i, &pos1)| {
                    let mut interactions = Vec::new();
                    for (j, &pos2) in agent_positions.iter().enumerate() {
                        if i != j {
                            let distance =
                                ((pos1.0 - pos2.0).powi(2) + (pos1.1 - pos2.1).powi(2)).sqrt();
                            if distance < 50.0 {
                                // Much simpler interaction calculation
                                let force = 1.0 / (distance * distance + 1.0);
                                let angle = (pos2.1 - pos1.1).atan2(pos2.0 - pos1.0);
                                let fx = force * angle.cos();
                                let fy = force * angle.sin();
                                interactions.push((j, fx, fy));
                            }
                        }
                    }
                    interactions
                })
                .collect();

            // Apply interaction forces in parallel - much smaller forces
            self.agents
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, agent)| {
                    if i < interaction_matrix.len() {
                        for (_, fx, fy) in &interaction_matrix[i] {
                            agent.dx += fx * 0.0001; // Reduced from 0.001
                            agent.dy += fy * 0.0001; // Reduced from 0.001
                        }
                    }
                });
        }

        // Simplified resource distribution calculations
        if num_resources > 0 {
            let resource_energy: Vec<_> = self.resources.par_iter().map(|r| r.energy).collect();
            let total_energy: f64 = resource_energy.par_iter().sum();
            let average_energy = total_energy / num_resources as f64;

            // Only apply redistribution if variance is very high
            let energy_variance: f64 = resource_energy
                .par_iter()
                .map(|&energy| (energy - average_energy).powi(2))
                .sum::<f64>()
                / num_resources as f64;

            if energy_variance > 10.0 {
                // Increased threshold
                self.resources.par_iter_mut().for_each(|resource| {
                    let energy_diff = resource.energy - average_energy;
                    resource.energy += energy_diff * 0.001; // Reduced from 0.01
                });
            }
        }

        // Much simplified environmental calculations
        if num_agents > 0 {
            let environmental_factors: Vec<_> = (0..num_agents.min(100)) // Limit to 100 agents
                .into_par_iter()
                .map(|i| {
                    let x = (i as f64 * 0.1) % self.width;
                    let y = (i as f64 * 0.1) % self.height;
                    let time_factor = self.time * 0.0001; // Reduced from 0.001
                    let noise = (x * 0.001 + time_factor).sin() * (y * 0.001 + time_factor).cos(); // Much reduced frequency
                    noise
                })
                .collect();

            // Apply environmental effects in parallel - much smaller effect
            self.agents
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, agent)| {
                    if i < environmental_factors.len() {
                        let factor = environmental_factors[i];
                        agent.energy += factor * 0.00001; // Reduced from 0.0001 - much smaller effect
                    }
                });
        }
    }

    fn spawn_resource(&mut self) {
        let mut rng = thread_rng();
        let x = rng.gen_range(0.0..self.width);
        let y = rng.gen_range(0.0..self.height);
        let resource = Resource::new(x, y);
        self.resources.push(resource);
    }

    fn cleanup_dead_agents(&mut self) {
        self.agents.retain(|agent| agent.is_alive());
    }

    pub fn add_agent(&mut self, x: f64, y: f64) {
        if self.agents.len() < self.max_agents {
            let genes = Genes::new();
            let agent = Agent::new(x, y, genes, 0);
            self.agents.push(agent);
        }
    }

    pub fn add_resource(&mut self, x: f64, y: f64) {
        if self.resources.len() < self.max_resources {
            let resource = Resource::new(x, y);
            self.resources.push(resource);
        }
    }

    pub fn reset(&mut self) {
        self.agents.clear();
        self.resources.clear();
        self.time = 0.0;
        self.resource_spawn_timer = 0.0;
        self.spawn_initial_population();
    }

    pub fn get_stats(&self) -> SimulationStats {
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

        // Calculate statistics in parallel
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
        ) = self
            .agents
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
                 agent| {
                    let agent_fitness = agent.energy
                        * agent.genes.speed
                        * agent.genes.size
                        * agent.genes.aggression;
                    (
                        energy + agent.energy,
                        age + agent.age,
                        speed + agent.genes.speed,
                        size + agent.genes.size,
                        aggression + agent.genes.aggression,
                        sense_range + agent.genes.sense_range,
                        efficiency + agent.genes.energy_efficiency,
                        kills + agent.kills,
                        gen.max(agent.generation),
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
            );

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
}
