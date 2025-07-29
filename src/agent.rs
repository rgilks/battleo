use crate::genes::Genes;
use crate::resource::Resource;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub energy: f64,
    pub max_energy: f64,
    pub age: f64,
    pub genes: Genes,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub state: AgentState,
    pub last_reproduction: f64,
    pub kills: u32,
    pub generation: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AgentState {
    Seeking,
    Hunting,
    Feeding,
    Reproducing,
    Fighting,
    Fleeing,
}

impl Agent {
    pub fn new(x: f64, y: f64, genes: Genes, generation: u32) -> Self {
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);

        Self {
            x,
            y,
            dx: angle.cos() * genes.speed,
            dy: angle.sin() * genes.speed,
            energy: 80.0, // Increased from 50.0 - better starting energy
            max_energy: 100.0,
            age: 0.0,
            genes,
            target_x: None,
            target_y: None,
            state: AgentState::Seeking,
            last_reproduction: 0.0,
            kills: 0,
            generation,
        }
    }

    pub fn update(
        &mut self,
        delta_time: f64,
        resources: &[Resource],
        agents: &[Agent],
        canvas_width: f64,
        canvas_height: f64,
    ) -> Option<usize> {
        self.age += delta_time;

        // Much reduced energy consumption - more sustainable
        let energy_cost = (self.genes.size * 0.01 + self.genes.speed * 0.005) * delta_time; // Reduced from 0.1 and 0.05
        let environmental_factor = 1.0 + (self.x / canvas_width + self.y / canvas_height) * 0.001; // Reduced from 0.01
        self.energy -= energy_cost / self.genes.energy_efficiency * environmental_factor;

        // Death from old age or no energy
        if self.energy <= 0.0 || self.age > 2000.0 {
            // Increased max age from 1000
            return None;
        }

        // Complex behavioral decision making
        self.update_behavior_state(resources, agents);

        let mut consumed_resource = None;

        // Update behavior based on current state
        match self.state {
            AgentState::Seeking => self.seek_targets(resources, agents),
            AgentState::Hunting => self.hunt_target(delta_time),
            AgentState::Feeding => consumed_resource = self.feed_on_resource(resources),
            AgentState::Reproducing => self.reproduce(),
            AgentState::Fighting => self.fight_agent(agents),
            AgentState::Fleeing => self.flee_from_danger(delta_time),
        }

        // Move agent with complex physics
        self.move_agent(delta_time, canvas_width, canvas_height);

        // Check for reproduction with more complex conditions
        if self.can_reproduce() {
            self.state = AgentState::Reproducing;
        }

        // Reduced learning calculations - only run occasionally
        if self.age % 1.0 < delta_time {
            // Only run every 1 second instead of every 10 seconds (10x faster)
            self.perform_learning_calculations(delta_time);
        }

        consumed_resource
    }

    fn update_behavior_state(&mut self, resources: &[Resource], agents: &[Agent]) {
        // Complex decision making based on environment
        let mut threat_level = 0.0;
        let mut resource_abundance = 0.0;
        let mut _population_density = 0.0;

        // Calculate environmental factors
        for agent in agents {
            if agent.id() != self.id() {
                let distance = self.distance_to(agent.x, agent.y);
                if distance < 100.0 {
                    _population_density += 1.0 / (distance + 1.0);
                    if agent.genes.size > self.genes.size * 1.2 {
                        threat_level += 1.0 / (distance + 1.0);
                    }
                }
            }
        }

        for resource in resources {
            let distance = resource.distance_to(self.x, self.y);
            if distance < 200.0 {
                resource_abundance += resource.energy / (distance + 1.0);
            }
        }

        // Complex behavioral adaptation - REMOVED SPEED MULTIPLICATION
        // This was causing exponential speed growth
        let _stress_factor = threat_level * 0.5 + (1.0 - resource_abundance / 1000.0) * 0.3;
        // self.genes.speed *= (1.0 + stress_factor * 0.1).min(2.0); // REMOVED THIS LINE
    }

    fn perform_learning_calculations(&mut self, _delta_time: f64) {
        // Simplified learning calculations - much less expensive
        let input = self.energy / self.max_energy;
        let weight1 = self.genes.speed;
        let weight2 = self.genes.size;
        let weight3 = self.genes.aggression;

        let hidden1 = (input * weight1 + self.age * weight2).tanh();
        let hidden2 = (hidden1 * weight3 + self.energy * weight1).tanh();
        let output = 1.0 / (1.0 + (-hidden2 * weight2 - input * weight3).exp()); // sigmoid function

        // Apply learning to genes (subtle adaptation) - only if output is high
        // REMOVED GENE MODIFICATIONS - these were causing instability
        if output > 0.7 {
            // self.genes.speed *= 1.0001; // REMOVED
            // self.genes.size *= 1.0001; // REMOVED
        }
    }

    fn seek_targets(&mut self, resources: &[Resource], agents: &[Agent]) {
        let mut best_target = None;
        let mut best_score = f64::NEG_INFINITY;

        // Look for resources
        for resource in resources {
            if resource.is_available() {
                let distance = resource.distance_to(self.x, self.y);
                if distance <= self.genes.sense_range {
                    let score = resource.energy / (distance + 1.0);
                    if score > best_score {
                        best_score = score;
                        best_target = Some((resource.x, resource.y, false));
                    }
                }
            }
        }

        // Look for other agents (potential prey or threats)
        for agent in agents {
            if agent.id() != self.id() {
                let distance = self.distance_to(agent.x, agent.y);
                if distance <= self.genes.sense_range {
                    let size_ratio = agent.genes.size / self.genes.size;
                    let energy_ratio = agent.energy / self.energy;

                    // Decide whether to hunt or flee
                    if size_ratio < 0.7 && energy_ratio > 0.5 && self.genes.aggression > 0.3 {
                        // Hunt smaller agent with good energy
                        let score = agent.energy / (distance + 1.0) * self.genes.aggression;
                        if score > best_score {
                            best_score = score;
                            best_target = Some((agent.x, agent.y, true));
                        }
                    } else if size_ratio > 1.3 && self.genes.aggression < 0.5 {
                        // Flee from larger agent
                        self.state = AgentState::Fleeing;
                        self.target_x = Some(self.x - (agent.x - self.x));
                        self.target_y = Some(self.y - (agent.y - self.y));
                        return;
                    }
                }
            }
        }

        if let Some((tx, ty, is_agent)) = best_target {
            self.target_x = Some(tx);
            self.target_y = Some(ty);
            self.state = if is_agent {
                AgentState::Hunting
            } else {
                AgentState::Hunting
            };
        } else {
            // Random movement if no targets
            self.random_movement();
        }
    }

    fn hunt_target(&mut self, _delta_time: f64) {
        if let (Some(tx), Some(ty)) = (self.target_x, self.target_y) {
            let dx = tx - self.x;
            let dy = ty - self.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 5.0 {
                // Close enough to interact
                self.state = AgentState::Feeding;
            } else {
                // Move towards target
                let speed = self.genes.speed * 2.0;
                self.dx = (dx / distance) * speed;
                self.dy = (dy / distance) * speed;
            }
        } else {
            self.state = AgentState::Seeking;
        }
    }

    fn feed_on_resource(&mut self, resources: &[Resource]) -> Option<usize> {
        if let (Some(_tx), Some(_ty)) = (self.target_x, self.target_y) {
            for (i, resource) in resources.iter().enumerate() {
                if resource.distance_to(self.x, self.y) < 5.0 {
                    // Consume the resource and gain energy - much more energy from resources
                    self.energy += 20.0 * self.genes.energy_efficiency; // Increased from 5.0
                    if self.energy > self.max_energy {
                        self.energy = self.max_energy;
                    }

                    // Reset state
                    self.state = AgentState::Seeking;
                    self.target_x = None;
                    self.target_y = None;

                    return Some(i);
                }
            }
        }

        // If we can't find the target resource, go back to seeking
        self.state = AgentState::Seeking;
        self.target_x = None;
        self.target_y = None;

        None
    }

    fn fight_agent(&mut self, agents: &[Agent]) {
        if let (Some(_tx), Some(_ty)) = (self.target_x, self.target_y) {
            for agent in agents {
                if agent.distance_to(self.x, self.y) < 5.0 {
                    // Combat mechanics
                    let my_power = self.genes.size * self.genes.aggression * self.energy;
                    let their_power = agent.genes.size * agent.genes.aggression * agent.energy;

                    if my_power > their_power * 1.2 {
                        // Win the fight
                        self.energy += agent.energy * 0.5;
                        self.kills += 1;
                    } else {
                        // Lose the fight
                        self.energy *= 0.7;
                        self.state = AgentState::Fleeing;
                    }
                    break;
                }
            }
        }
        self.state = AgentState::Seeking;
        self.target_x = None;
        self.target_y = None;
    }

    fn flee_from_danger(&mut self, _delta_time: f64) {
        if let (Some(tx), Some(ty)) = (self.target_x, self.target_y) {
            let dx = tx - self.x;
            let dy = ty - self.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > self.genes.sense_range {
                self.state = AgentState::Seeking;
                self.target_x = None;
                self.target_y = None;
            } else {
                let speed = self.genes.speed * 3.0; // Faster when fleeing
                self.dx = (dx / distance) * speed;
                self.dy = (dy / distance) * speed;
            }
        }
    }

    fn reproduce(&mut self) {
        // Reproduction costs energy - higher cost to prevent overpopulation
        self.energy *= 0.7; // Increased from 0.9 - more punishing
        self.last_reproduction = self.age;
        self.state = AgentState::Seeking;
    }

    fn move_agent(&mut self, delta_time: f64, canvas_width: f64, canvas_height: f64) {
        // Apply movement
        self.x += self.dx * delta_time;
        self.y += self.dy * delta_time;

        // Boundary wrapping - these will be set by the simulation
        // For now, use reasonable defaults
        let max_x = canvas_width;
        let max_y = canvas_height;
        if self.x < 0.0 {
            self.x = max_x;
        }
        if self.x > max_x {
            self.x = 0.0;
        }
        if self.y < 0.0 {
            self.y = max_y;
        }
        if self.y > max_y {
            self.y = 0.0;
        }

        // Add some randomness to movement
        let mut rng = thread_rng();
        if rng.gen::<f64>() < 0.01 {
            let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
            self.dx += angle.cos() * 0.1;
            self.dy += angle.sin() * 0.1;
        }

        // Normalize direction vector
        let length = (self.dx * self.dx + self.dy * self.dy).sqrt();
        if length > 0.0 {
            self.dx /= length;
            self.dy /= length;
        }
    }

    fn random_movement(&mut self) {
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        self.dx = angle.cos() * self.genes.speed;
        self.dy = angle.sin() * self.genes.speed;
    }

    pub fn can_reproduce(&self) -> bool {
        self.energy > 20.0 && self.age > 5.0 && self.age - self.last_reproduction > 3.0
        // Much faster reproduction (10x faster)
    }

    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn id(&self) -> u64 {
        // Simple ID based on position and generation
        ((self.x * 1000.0) as u64) ^ ((self.y * 1000.0) as u64) ^ (self.generation as u64)
    }

    pub fn is_alive(&self) -> bool {
        self.energy > 0.0 && self.age < 1000.0
    }

    pub fn create_offspring(&self, other: &Agent) -> Self {
        let new_genes = self
            .genes
            .inherit_from(&other.genes, self.genes.mutation_rate);
        let mut rng = thread_rng();

        // Position offspring near parent
        let offset_x = rng.gen_range(-10.0..10.0);
        let offset_y = rng.gen_range(-10.0..10.0);

        Self::new(
            self.x + offset_x,
            self.y + offset_y,
            new_genes,
            self.generation + 1,
        )
    }
}
