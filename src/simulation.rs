use crate::agent::Agent;
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

pub struct Simulation {
    pub agents: Vec<Agent>,
    pub resources: Vec<Resource>,
    pub width: f64,
    pub height: f64,
    pub time: f64,
    pub resource_spawn_timer: f64,
    pub max_agents: usize,
    pub max_resources: usize,
    grid_cell_size: f64,
    grid_width: usize,
    grid_height: usize,
    spatial_grid: Vec<Vec<Vec<usize>>>, // Grid of agent indices
}

impl Simulation {
    pub fn is_rayon_available() -> bool {
        unsafe { THREAD_POOL_AVAILABLE && RAYON_INITIALIZED }
    }

    pub fn set_rayon_initialized(initialized: bool) {
        unsafe { RAYON_INITIALIZED = initialized; }
    }

    pub fn new(width: f64, height: f64) -> Self {
        // Note: Thread pool initialization is handled separately via the ParallelProcessor
        // This prevents recursive initialization issues during simulation construction

        // Initialize spatial grid for efficient neighbor lookups
        let grid_cell_size = 50.0; // Cell size for spatial partitioning
        let grid_width = (width / grid_cell_size).ceil() as usize;
        let grid_height = (height / grid_cell_size).ceil() as usize;
        let spatial_grid = vec![vec![Vec::new(); grid_height]; grid_width];

        let mut simulation = Self {
            agents: Vec::new(),
            resources: Vec::new(),
            width,
            height,
            time: 0.0,
            resource_spawn_timer: 0.0,
            max_agents: 10000,   // Increased from 5000 - twice as many agents
            max_resources: 1500, // Reduced from 3000 - less food to prevent overcrowding
            grid_cell_size,
            grid_width,
            grid_height,
            spatial_grid,
        };

        // Initialize with some agents and resources
        simulation.spawn_initial_population();

        simulation
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
        let grid_radius = (radius / self.grid_cell_size).ceil() as usize;

        for dx in -(grid_radius as i32)..=(grid_radius as i32) {
            for dy in -(grid_radius as i32)..=(grid_radius as i32) {
                let grid_x = (center_x as i32 + dx) as usize;
                let grid_y = (center_y as i32 + dy) as usize;

                if grid_x < self.grid_width && grid_y < self.grid_height {
                    nearby.extend(&self.spatial_grid[grid_x][grid_y]);
                }
            }
        }
        nearby
    }

    fn update_spatial_grid(&mut self) {
        // Clear the grid
        for x in 0..self.grid_width {
            for y in 0..self.grid_height {
                self.spatial_grid[x][y].clear();
            }
        }

        // Populate the grid with agent indices
        for (i, agent) in self.agents.iter().enumerate() {
            let (grid_x, grid_y) = self.get_grid_position(agent.x, agent.y);
            self.spatial_grid[grid_x][grid_y].push(i);
        }
    }

    fn spawn_initial_population(&mut self) {
        let mut rng = thread_rng();

        // Spawn initial agents - increased for more activity
        let initial_agents = (self.max_agents as f64 * 0.1) as usize; // Increased from 0.05 to 0.1 (10% of max)
        for _ in 0..initial_agents {
            let x = rng.gen_range(0.0..self.width);
            let y = rng.gen_range(0.0..self.height);
            let genes = Genes::new();
            let agent = Agent::new(x, y, genes, 0);
            self.agents.push(agent);
        }

        // Spawn initial resources - reduced to prevent overcrowding
        let initial_resources = (self.max_resources as f64 * 0.1) as usize; // Reduced from 0.2 to 0.1 (10% of max)
        for _ in 0..initial_resources {
            let x = rng.gen_range(0.0..self.width);
            let y = rng.gen_range(0.0..self.height);
            let resource = Resource::new(x, y);
            self.resources.push(resource);
        }
    }

    pub fn update(&mut self) {
        let delta_time = 1.0 / 60.0; // Much faster simulation (60 FPS instead of 12 FPS)
        self.time += delta_time;

        // Update spatial grid for efficient neighbor lookups
        self.update_spatial_grid();

        // Update resources
        if Self::is_rayon_available() {
            self.resources.par_iter_mut().for_each(|resource| {
                resource.update(delta_time);
            });
        } else {
            for resource in self.resources.iter_mut() {
                resource.update(delta_time);
            }
        }

        // Spawn new resources at a reasonable rate
        self.resource_spawn_timer += delta_time;
        if self.resource_spawn_timer > 0.5 && self.resources.len() < self.max_resources {
            // Much slower spawning (every 0.5 seconds instead of 0.01)
            self.spawn_resource();
            self.resource_spawn_timer = 0.0;
        }

        // Update agents
        if Self::is_rayon_available() {
            let resources = Arc::new(self.resources.clone());
            let agent_updates: Vec<_> = self
                .agents
                .par_iter_mut()
                .enumerate()
                .map(|(i, agent)| {
                    let consumed =
                        agent.update(delta_time, &resources, &[], self.width, self.height);
                    (i, consumed)
                })
                .collect();

            // Mark consumed resources for depletion
            for (_, consumed) in agent_updates {
                if let Some(index) = consumed {
                    if index < self.resources.len() {
                        self.resources[index].is_depleting = true;
                        self.resources[index].deplete_fade = 0.0;
                    }
                }
            }
        } else {
            for agent in self.agents.iter_mut() {
                let consumed =
                    agent.update(delta_time, &self.resources, &[], self.width, self.height);
                if let Some(index) = consumed {
                    if index < self.resources.len() {
                        self.resources[index].is_depleting = true;
                        self.resources[index].deplete_fade = 0.0;
                    }
                }
            }
        }

        // Handle reproduction in parallel
        let safe_population_threshold = (self.max_agents as f64 * 0.8) as usize;

        if self.agents.len() >= safe_population_threshold {
            return; // Don't reproduce if population is too high
        }

        let population_growth_rate = self.agents.len() as f64 / self.max_agents as f64;
        if population_growth_rate > 0.6 {
            return; // Don't reproduce if population is already substantial
        }

        // Handle reproduction
        if Self::is_rayon_available() {
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
                                    && other.energy > 50.0
                                    && other.can_reproduce()
                            })
                            .collect();

                        if let Some(mate) = potential_mates.choose(&mut thread_rng()) {
                            // Additional check: only reproduce if we won't exceed max agents
                            if self.agents.len() < self.max_agents {
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

            self.agents.extend(reproduction_results);
        } else {
            let mut new_agents = Vec::new();
            let mut rng = thread_rng();
            for agent in self.agents.iter() {
                if agent.can_reproduce() {
                    // Find a suitable mate nearby
                    let potential_mates: Vec<_> = self
                        .agents
                        .iter()
                        .filter(|other| {
                            other.id() != agent.id()
                                && agent.distance_to(other.x, other.y) < 20.0
                                && other.energy > 50.0
                                && other.can_reproduce()
                        })
                        .collect();

                    if let Some(mate) = potential_mates.choose(&mut rng) {
                        // Additional check: only reproduce if we won't exceed max agents
                        if self.agents.len() + new_agents.len() < self.max_agents {
                            // Random chance to reproduce (50% chance)
                            if rng.gen::<f64>() < 0.5 {
                                new_agents.push(agent.create_offspring(mate));
                            }
                        }
                    }
                }
            }
            self.agents.extend(new_agents);
        }

        // Clean up dead agents
        self.agents.retain(|agent| agent.is_alive());

        // Clean up fully depleted resources
        self.resources
            .retain(|resource| !resource.is_depleting || resource.deplete_fade < 1.0);

        // Add some complex calculations to utilize CPU cores
        self.perform_complex_calculations();
    }

    fn update_resources_sequential(&mut self, delta_time: f64) {
        // Sequential resource updates
        for resource in self.resources.iter_mut() {
            resource.update(delta_time);
        }
    }

    fn update_agents_sequential(&mut self, delta_time: f64) {
        // Sequential agent processing
        let mut consumed_indices = Vec::new();

        for agent in self.agents.iter_mut() {
            let consumed = agent.update(delta_time, &self.resources, &[], self.width, self.height);
            if let Some(index) = consumed {
                consumed_indices.push(index);
            }
        }

        // Mark consumed resources for depletion (let them fade out instead of removing immediately)
        for &index in &consumed_indices {
            if index < self.resources.len() {
                self.resources[index].is_depleting = true;
                self.resources[index].deplete_fade = 0.0;
            }
        }
    }

    fn handle_reproduction_parallel(&mut self) {
        // Use Rayon to handle reproduction in parallel
        let mut new_agents = Vec::new();

        // More permissive population control for more agents
        let safe_population_threshold = (self.max_agents as f64 * 0.8) as usize; // Increased from 0.6

        if self.agents.len() >= safe_population_threshold {
            return; // Don't reproduce if population is too high
        }

        // More permissive reproduction conditions
        let population_growth_rate = self.agents.len() as f64 / self.max_agents as f64;
        if population_growth_rate > 0.6 {
            return; // Don't reproduce if population is already substantial
        }

        // Process reproduction sequentially
        let mut rng = thread_rng();
        for agent in self.agents.iter() {
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

                if let Some(mate) = potential_mates.choose(&mut rng) {
                    // Additional check: only reproduce if we won't exceed max agents
                    if self.agents.len() + new_agents.len() < self.max_agents {
                        // Random chance to reproduce (50% chance)
                        if rng.gen::<f64>() < 0.5 {
                            new_agents.push(agent.create_offspring(mate));
                        }
                    }
                }
            }
        }

        self.agents.extend(new_agents);
    }

    fn perform_complex_calculations(&mut self) {
        // Efficient parallel calculations using spatial grid
        let num_agents = self.agents.len();
        let num_resources = self.resources.len();

        // Only perform calculations if we have enough entities
        if num_agents > 10 && num_resources > 10 {
            // Use spatial grid for efficient neighbor lookups - O(n) instead of O(n²)
            // Collect all agent positions and nearby indices first to avoid borrowing issues
            let agent_positions: Vec<_> = self
                .agents
                .iter()
                .map(|a| (a.x, a.y, a.energy, a.genes.speed))
                .collect();
            let nearby_indices: Vec<_> = self
                .agents
                .iter()
                .map(|agent| self.get_nearby_agents(agent.x, agent.y, 100.0))
                .collect();

            self.agents
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, agent)| {
                    let mut total_fx = 0.0;
                    let mut total_fy = 0.0;

                    if i < nearby_indices.len() {
                        for &j in &nearby_indices[i] {
                            if i != j && j < agent_positions.len() {
                                let (other_x, other_y, other_energy, other_speed) =
                                    agent_positions[j];
                                let distance = ((agent.x - other_x).powi(2)
                                    + (agent.y - other_y).powi(2))
                                .sqrt();

                                if distance < 100.0 && distance > 0.0 {
                                    // Efficient interaction calculation
                                    let force = 1.0 / (distance * distance + 1.0);
                                    let energy_factor = (agent.energy + other_energy) / 200.0;
                                    let speed_factor = (agent.genes.speed + other_speed) / 2.0;
                                    let combined_force = force * energy_factor * speed_factor;

                                    let angle = (other_y - agent.y).atan2(other_x - agent.x);
                                    let fx = combined_force * angle.cos();
                                    let fy = combined_force * angle.sin();

                                    // Simple interaction types
                                    let repulsion =
                                        if distance < 10.0 { 1.0 / distance } else { 0.0 };
                                    let attraction = if distance > 20.0 && distance < 80.0 {
                                        0.1 / distance
                                    } else {
                                        0.0
                                    };

                                    total_fx += fx * 0.001 + repulsion * 0.01 - attraction * 0.005;
                                    total_fy += fy * 0.001 + repulsion * 0.01 - attraction * 0.005;
                                }
                            }
                        }
                    }

                    // Apply accumulated forces
                    agent.dx += total_fx;
                    agent.dy += total_fy;
                });
        }

        // Enhanced resource distribution calculations with more work
        if num_resources > 0 {
            let resource_data: Vec<_> = self
                .resources
                .par_iter()
                .map(|r| (r.energy, r.x, r.y, r.size))
                .collect();
            let total_energy: f64 = resource_data
                .par_iter()
                .map(|(energy, _, _, _)| energy)
                .sum();
            let average_energy = total_energy / num_resources as f64;

            // Calculate efficient resource statistics in parallel
            let resource_stats: Vec<_> = resource_data
                .par_iter()
                .map(|(energy, x, y, size)| {
                    let energy_variance = (energy - average_energy).powi(2);
                    let spatial_factor = ((x * x + y * y).sqrt() / 1000.0).sin();
                    let size_factor = size / 10.0;
                    (energy_variance, spatial_factor, size_factor)
                })
                .collect();

            let total_variance: f64 = resource_stats
                .par_iter()
                .map(|(variance, _, _)| variance)
                .sum::<f64>()
                / num_resources as f64;
            let spatial_balance: f64 = resource_stats
                .par_iter()
                .map(|(_, spatial, _)| spatial)
                .sum::<f64>()
                / num_resources as f64;
            let size_balance: f64 = resource_stats
                .par_iter()
                .map(|(_, _, size)| size)
                .sum::<f64>()
                / num_resources as f64;

            // Apply enhanced redistribution based on multiple factors
            if total_variance > 5.0 || spatial_balance.abs() > 0.1 {
                self.resources.par_iter_mut().for_each(|resource| {
                    let energy_diff = resource.energy - average_energy;
                    let spatial_factor =
                        ((resource.x * resource.x + resource.y * resource.y).sqrt() / 1000.0).sin();
                    let redistribution = energy_diff * 0.01 + spatial_factor * 0.1;
                    resource.energy += redistribution;
                });
            }
        }

        // Enhanced environmental calculations with more work
        if num_agents > 0 {
            // Calculate environmental factors for all agents, not just 100
            let environmental_factors: Vec<_> = (0..num_agents)
                .into_par_iter()
                .map(|i| {
                    let x = (i as f64 * 0.1) % self.width;
                    let y = (i as f64 * 0.1) % self.height;
                    let time_factor = self.time * 0.001;

                    // Multiple environmental calculations
                    let noise1 = (x * 0.01 + time_factor).sin() * (y * 0.01 + time_factor).cos();
                    let noise2 = (x * 0.005 + time_factor * 0.5).cos()
                        * (y * 0.005 + time_factor * 0.5).sin();
                    let noise3 =
                        (x * 0.02 + time_factor * 2.0).sin() * (y * 0.02 + time_factor * 2.0).sin();

                    let combined_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
                    combined_noise
                })
                .collect();

            // Apply enhanced environmental effects in parallel
            self.agents
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, agent)| {
                    if i < environmental_factors.len() {
                        let factor = environmental_factors[i];
                        agent.energy += factor * 0.0001; // Increased effect

                        // Also affect movement slightly
                        agent.dx += factor * 0.00001;
                        agent.dy += factor * 0.00001;
                    }
                });
        }

        // Additional parallel work: genetic analysis and fitness calculations
        if num_agents > 0 {
            let fitness_data: Vec<_> = self
                .agents
                .par_iter()
                .map(|agent| {
                    let fitness = agent.genes.get_fitness_score();
                    let age_factor = agent.age / 1000.0;
                    let energy_factor = agent.energy / 100.0;
                    let generation_factor = agent.generation as f64;

                    (fitness, age_factor, energy_factor, generation_factor)
                })
                .collect();

            // Calculate population statistics in parallel
            let total_fitness: f64 = fitness_data
                .par_iter()
                .map(|(fitness, _, _, _)| fitness)
                .sum();
            let average_age: f64 = fitness_data
                .par_iter()
                .map(|(_, age, _, _)| age)
                .sum::<f64>()
                / num_agents as f64;
            let average_energy: f64 = fitness_data
                .par_iter()
                .map(|(_, _, energy, _)| energy)
                .sum::<f64>()
                / num_agents as f64;
            let max_generation: f64 = fitness_data
                .par_iter()
                .map(|(_, _, _, gen)| *gen)
                .reduce(|| 0.0, |a, b| a.max(b));

            // Apply population-wide effects based on statistics
            if total_fitness > 0.0 {
                let fitness_factor = total_fitness / (num_agents as f64 * 10.0);
                self.agents.par_iter_mut().for_each(|agent| {
                    agent.energy += fitness_factor * 0.001;
                });
            }
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

    fn cleanup_depleted_resources(&mut self) {
        // Remove resources that have fully faded out
        self.resources
            .retain(|resource| !resource.is_depleting || resource.deplete_fade < 1.0);
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
            self.agents
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
                )
        } else {
            // Sequential processing for WebAssembly compatibility
            self.agents.iter().fold(
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
}
