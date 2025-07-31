use hecs::{Entity, Query, World};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Energy {
    pub current: f64,
    pub max: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Age {
    pub value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genes {
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
pub struct AgentState {
    pub state: AgentStateEnum,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub last_reproduction: f64,
    pub kills: u32,
    pub generation: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AgentStateEnum {
    Seeking,
    Hunting,
    Feeding,
    Reproducing,
    Fighting,
    Fleeing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeathAnimation {
    pub fade: f64,
    pub reason: DeathReason,
    pub is_dying: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeathReason {
    Starvation,
    OldAge,
    KilledByPredator,
    Combat,
    NaturalCauses,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnAnimation {
    pub fade: f64,
    pub spawn_position: Option<(f64, f64)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Size {
    pub value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTag;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceTag;

// ============================================================================
// WORLD MANAGEMENT
// ============================================================================

pub struct EcsWorld {
    pub world: World,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub max_agents: usize,
    pub max_resources: usize,
    pub resource_spawn_timer: f64,
}

impl EcsWorld {
    pub fn new(canvas_width: f64, canvas_height: f64) -> Self {
        let world = World::new();

        let mut ecs_world = Self {
            world,
            canvas_width,
            canvas_height,
            max_agents: 10000,
            max_resources: 1500,
            resource_spawn_timer: 0.0,
        };

        // Spawn initial population
        ecs_world.spawn_initial_population();

        ecs_world
    }

    pub fn update(&mut self) {
        let delta_time = 1.0 / 60.0;

        // Update resources
        self.update_resources(delta_time);

        // Update agents
        self.update_agents(delta_time);

        // Handle death
        self.handle_death();

        // Handle reproduction
        self.handle_reproduction();

        // Spawn new resources
        self.resource_spawn_timer += delta_time;
        if self.resource_spawn_timer > 0.5 && self.get_resource_count() < self.max_resources {
            self.spawn_resource();
            self.resource_spawn_timer = 0.0;
        }
    }

    fn update_resources(&mut self, delta_time: f64) {
        // Sequential processing for now
        for (_, resource) in self.world.query_mut::<&mut Resource>() {
            resource.update(delta_time);
        }
    }

    fn update_agents(&mut self, delta_time: f64) {
        // Get all resources for agent decision making
        let resources: Vec<_> = self
            .world
            .query::<(&Position, &Resource)>()
            .iter()
            .map(|(_, (pos, res))| (pos.x, pos.y, res.clone()))
            .collect();

        let resources = Arc::new(resources);
        let canvas_width = self.canvas_width;
        let canvas_height = self.canvas_height;

        // Sequential processing for now
        for (_, (pos, vel, energy, age, state, genes)) in self.world.query_mut::<(
            &mut Position,
            &mut Velocity,
            &mut Energy,
            &mut Age,
            &mut AgentState,
            &Genes,
        )>() {
            // Inline the update logic to avoid borrowing issues
            age.value += delta_time;

            // Energy consumption
            let base_energy_cost = (genes.size * 0.05 + genes.speed * 0.02) * delta_time;
            let metabolism_factor = genes.metabolism;
            let environmental_factor = 1.0 + (pos.x / canvas_width + pos.y / canvas_height) * 0.001;
            let total_energy_cost = base_energy_cost * metabolism_factor * environmental_factor;
            energy.current -= total_energy_cost / genes.energy_efficiency;

            // Check for death
            if energy.current <= 0.0 || age.value > 200.0 {
                continue;
            }

            // Update behavior - inline to avoid borrowing issues
            // Simple seeking behavior
            let mut best_target = None;
            let mut best_score = f64::NEG_INFINITY;

            for (rx, ry, resource) in resources.iter() {
                if resource.is_available() {
                    let distance = ((pos.x - rx).powi(2) + (pos.y - ry).powi(2)).sqrt();
                    if distance <= genes.sense_range {
                        let score = resource.energy / (distance + 1.0);
                        if score > best_score {
                            best_score = score;
                            best_target = Some((*rx, *ry));
                        }
                    }
                }
            }

            if let Some((tx, ty)) = best_target {
                state.target_x = Some(tx);
                state.target_y = Some(ty);
                state.state = AgentStateEnum::Hunting;
            } else {
                // Random movement
                let mut rng = thread_rng();
                let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
                vel.dx = angle.cos() * genes.speed;
                vel.dy = angle.sin() * genes.speed;
            }

            // Move agent - inline to avoid borrowing issues
            // Apply movement
            pos.x += vel.dx * delta_time;
            pos.y += vel.dy * delta_time;

            // Boundary wrapping
            if pos.x < 0.0 {
                pos.x = canvas_width;
            }
            if pos.x > canvas_width {
                pos.x = 0.0;
            }
            if pos.y < 0.0 {
                pos.y = canvas_height;
            }
            if pos.y > canvas_height {
                pos.y = 0.0;
            }

            // Normalize direction vector
            let length = (vel.dx * vel.dx + vel.dy * vel.dy).sqrt();
            if length > 0.0 {
                vel.dx /= length;
                vel.dy /= length;
            }
        }
    }

    fn update_agent(
        &self,
        pos: &mut Position,
        vel: &mut Velocity,
        energy: &mut Energy,
        age: &mut Age,
        state: &mut AgentState,
        genes: &Genes,
        resources: &Arc<Vec<(f64, f64, Resource)>>,
        delta_time: f64,
        canvas_width: f64,
        canvas_height: f64,
    ) {
        age.value += delta_time;

        // Energy consumption
        let base_energy_cost = (genes.size * 0.05 + genes.speed * 0.02) * delta_time;
        let metabolism_factor = genes.metabolism;
        let environmental_factor = 1.0 + (pos.x / canvas_width + pos.y / canvas_height) * 0.001;
        let total_energy_cost = base_energy_cost * metabolism_factor * environmental_factor;
        energy.current -= total_energy_cost / genes.energy_efficiency;

        // Check for death
        if energy.current <= 0.0 || age.value > 200.0 {
            // Mark for death - this will be handled by the death system
            return;
        }

        // Update behavior
        self.update_behavior(pos, vel, state, genes, resources);

        // Move agent
        self.move_agent(pos, vel, delta_time, canvas_width, canvas_height);
    }

    fn update_behavior(
        &self,
        pos: &Position,
        vel: &mut Velocity,
        state: &mut AgentState,
        genes: &Genes,
        resources: &Arc<Vec<(f64, f64, Resource)>>,
    ) {
        // Simple seeking behavior
        let mut best_target = None;
        let mut best_score = f64::NEG_INFINITY;

        for (rx, ry, resource) in resources.iter() {
            if resource.is_available() {
                let distance = ((pos.x - rx).powi(2) + (pos.y - ry).powi(2)).sqrt();
                if distance <= genes.sense_range {
                    let score = resource.energy / (distance + 1.0);
                    if score > best_score {
                        best_score = score;
                        best_target = Some((*rx, *ry));
                    }
                }
            }
        }

        if let Some((tx, ty)) = best_target {
            state.target_x = Some(tx);
            state.target_y = Some(ty);
            state.state = AgentStateEnum::Hunting;
        } else {
            // Random movement
            let mut rng = thread_rng();
            let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
            vel.dx = angle.cos() * genes.speed;
            vel.dy = angle.sin() * genes.speed;
        }
    }

    fn move_agent(
        &self,
        pos: &mut Position,
        vel: &mut Velocity,
        delta_time: f64,
        canvas_width: f64,
        canvas_height: f64,
    ) {
        // Apply movement
        pos.x += vel.dx * delta_time;
        pos.y += vel.dy * delta_time;

        // Boundary wrapping
        if pos.x < 0.0 {
            pos.x = canvas_width;
        }
        if pos.x > canvas_width {
            pos.x = 0.0;
        }
        if pos.y < 0.0 {
            pos.y = canvas_height;
        }
        if pos.y > canvas_height {
            pos.y = 0.0;
        }

        // Normalize direction vector
        let length = (vel.dx * vel.dx + vel.dy * vel.dy).sqrt();
        if length > 0.0 {
            vel.dx /= length;
            vel.dy /= length;
        }
    }

    fn handle_death(&mut self) {
        let mut to_remove = Vec::new();

        for (entity, (energy, age)) in self.world.query::<(&Energy, &Age)>().iter() {
            if energy.current <= 0.0 || age.value > 200.0 {
                to_remove.push(entity);
            }
        }

        // Remove dead entities
        for entity in to_remove {
            self.world.despawn(entity).ok();
        }
    }

    fn handle_reproduction(&mut self) {
        // Simplified reproduction - just spawn new agents occasionally
        let mut rng = thread_rng();
        let agent_count = self.get_agent_count();

        if agent_count < self.max_agents && rng.gen::<f64>() < 0.1 {
            let x = rng.gen_range(0.0..self.canvas_width);
            let y = rng.gen_range(0.0..self.canvas_height);
            let genes = self.generate_random_genes();
            self.spawn_agent(x, y, genes, 0);
        }
    }

    fn can_reproduce(&self, energy: &Energy, age: &Age, state: &AgentState) -> bool {
        energy.current > 10.0 && age.value > 2.0 && age.value - state.last_reproduction > 1.0
    }

    fn distance(&self, pos1: &Position, pos2: &Position) -> f64 {
        ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt()
    }

    fn inherit_genes(&self, genes1: &Genes, genes2: &Genes) -> Genes {
        let mut rng = thread_rng();
        let blend_factor = rng.gen_range(0.3..0.7);

        Genes {
            speed: genes1.speed * blend_factor + genes2.speed * (1.0 - blend_factor),
            sense_range: genes1.sense_range * blend_factor
                + genes2.sense_range * (1.0 - blend_factor),
            size: genes1.size * blend_factor + genes2.size * (1.0 - blend_factor),
            energy_efficiency: genes1.energy_efficiency * blend_factor
                + genes2.energy_efficiency * (1.0 - blend_factor),
            reproduction_threshold: genes1.reproduction_threshold * blend_factor
                + genes2.reproduction_threshold * (1.0 - blend_factor),
            mutation_rate: genes1.mutation_rate * blend_factor
                + genes2.mutation_rate * (1.0 - blend_factor),
            aggression: genes1.aggression * blend_factor + genes2.aggression * (1.0 - blend_factor),
            color_hue: genes1.color_hue * blend_factor + genes2.color_hue * (1.0 - blend_factor),
            is_predator: genes1.is_predator * blend_factor
                + genes2.is_predator * (1.0 - blend_factor),
            hunting_speed: genes1.hunting_speed * blend_factor
                + genes2.hunting_speed * (1.0 - blend_factor),
            attack_power: genes1.attack_power * blend_factor
                + genes2.attack_power * (1.0 - blend_factor),
            defense: genes1.defense * blend_factor + genes2.defense * (1.0 - blend_factor),
            stealth: genes1.stealth * blend_factor + genes2.stealth * (1.0 - blend_factor),
            pack_mentality: genes1.pack_mentality * blend_factor
                + genes2.pack_mentality * (1.0 - blend_factor),
            territory_size: genes1.territory_size * blend_factor
                + genes2.territory_size * (1.0 - blend_factor),
            metabolism: genes1.metabolism * blend_factor + genes2.metabolism * (1.0 - blend_factor),
            intelligence: genes1.intelligence * blend_factor
                + genes2.intelligence * (1.0 - blend_factor),
            stamina: genes1.stamina * blend_factor + genes2.stamina * (1.0 - blend_factor),
        }
    }

    fn spawn_agent(&mut self, x: f64, y: f64, genes: Genes, generation: u32) {
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let size_value = genes.size * 3.0;

        self.world.spawn((
            Position { x, y },
            Velocity {
                dx: angle.cos() * genes.speed,
                dy: angle.sin() * genes.speed,
            },
            Energy {
                current: 80.0,
                max: 100.0,
            },
            Age { value: 0.0 },
            genes,
            AgentState {
                state: AgentStateEnum::Seeking,
                target_x: None,
                target_y: None,
                last_reproduction: 0.0,
                kills: 0,
                generation,
            },
            DeathAnimation {
                fade: 0.0,
                reason: DeathReason::NaturalCauses,
                is_dying: false,
            },
            SpawnAnimation {
                fade: 0.0,
                spawn_position: Some((x, y)),
            },
            Size { value: size_value },
            AgentTag,
        ));
    }

    fn spawn_resource(&mut self) {
        let mut rng = thread_rng();
        let x = rng.gen_range(0.0..self.canvas_width);
        let y = rng.gen_range(0.0..self.canvas_height);

        let initial_energy = rng.gen_range(15.0..40.0);
        let max_energy = rng.gen_range(30.0..60.0);

        self.world.spawn((
            Position { x, y },
            Resource {
                energy: 0.0,
                max_energy,
                size: 3.0,
                growth_rate: rng.gen_range(0.1..0.5),
                regeneration_rate: rng.gen_range(0.02..0.1),
                age: 0.0,
                target_energy: initial_energy,
                is_spawning: true,
                spawn_fade: 0.0,
                is_depleting: false,
                deplete_fade: 0.0,
            },
            Size { value: 3.0 },
            ResourceTag,
        ));
    }

    fn spawn_initial_population(&mut self) {
        let mut rng = thread_rng();

        // Spawn initial agents
        let initial_agents = 100; // 10% of max
        for _ in 0..initial_agents {
            let x = rng.gen_range(0.0..self.canvas_width);
            let y = rng.gen_range(0.0..self.canvas_height);
            let genes = self.generate_random_genes();
            self.spawn_agent(x, y, genes, 0);
        }

        // Spawn initial resources
        let initial_resources = 150; // 10% of max
        for _ in 0..initial_resources {
            self.spawn_resource();
        }
    }

    fn generate_random_genes(&self) -> Genes {
        let mut rng = thread_rng();

        Genes {
            speed: rng.gen_range(0.8..1.5),
            sense_range: rng.gen_range(30.0..80.0),
            size: rng.gen_range(0.9..1.3),
            energy_efficiency: rng.gen_range(0.8..1.2),
            reproduction_threshold: rng.gen_range(60.0..120.0),
            mutation_rate: rng.gen_range(0.02..0.08),
            aggression: rng.gen_range(0.2..0.8),
            color_hue: rng.gen_range(0.0..360.0),
            is_predator: rng.gen_range(0.0..0.3),
            hunting_speed: rng.gen_range(1.0..2.0),
            attack_power: rng.gen_range(0.5..1.5),
            defense: rng.gen_range(0.5..1.5),
            stealth: rng.gen_range(0.0..1.0),
            pack_mentality: rng.gen_range(0.0..1.0),
            territory_size: rng.gen_range(50.0..150.0),
            metabolism: rng.gen_range(0.8..1.4),
            intelligence: rng.gen_range(0.5..1.5),
            stamina: rng.gen_range(0.5..1.5),
        }
    }

    pub fn get_agent_count(&self) -> usize {
        self.world.query::<&AgentTag>().iter().count()
    }

    pub fn get_resource_count(&self) -> usize {
        self.world.query::<&ResourceTag>().iter().count()
    }

    pub fn add_agent(&mut self, x: f64, y: f64) {
        if self.get_agent_count() < self.max_agents {
            let genes = self.generate_random_genes();
            self.spawn_agent(x, y, genes, 0);
        }
    }

    pub fn add_resource(&mut self, x: f64, y: f64) {
        if self.get_resource_count() < self.max_resources {
            let entity = self.world.spawn((
                Position { x, y },
                Resource {
                    energy: 0.0,
                    max_energy: 60.0,
                    size: 3.0,
                    growth_rate: 0.3,
                    regeneration_rate: 0.05,
                    age: 0.0,
                    target_energy: 30.0,
                    is_spawning: true,
                    spawn_fade: 0.0,
                    is_depleting: false,
                    deplete_fade: 0.0,
                },
                Size { value: 3.0 },
                ResourceTag,
            ));
        }
    }

    pub fn reset(&mut self) {
        self.world = World::new();
        self.resource_spawn_timer = 0.0;
        self.spawn_initial_population();
    }

    pub fn get_agents(&self) -> Vec<(Position, Velocity, Energy, Age, AgentState, Genes, Size)> {
        self.world
            .query::<(
                &Position,
                &Velocity,
                &Energy,
                &Age,
                &AgentState,
                &Genes,
                &Size,
            )>()
            .iter()
            .map(|(_, (pos, vel, energy, age, state, genes, size))| {
                (
                    pos.clone(),
                    vel.clone(),
                    energy.clone(),
                    age.clone(),
                    state.clone(),
                    genes.clone(),
                    size.clone(),
                )
            })
            .collect()
    }

    pub fn get_resources(&self) -> Vec<(Position, Resource, Size)> {
        self.world
            .query::<(&Position, &Resource, &Size)>()
            .iter()
            .map(|(_, (pos, resource, size))| (pos.clone(), resource.clone(), size.clone()))
            .collect()
    }
}

// Extension trait for Resource to add the update method
impl Resource {
    pub fn update(&mut self, delta_time: f64) {
        self.age += delta_time;

        // Handle spawning fade-in
        if self.is_spawning {
            self.spawn_fade += delta_time * 2.0;
            if self.spawn_fade >= 1.0 {
                self.spawn_fade = 1.0;
                self.is_spawning = false;
            }
        }

        // Handle depletion fade-out
        if self.is_depleting {
            self.deplete_fade += delta_time * 3.0;
            if self.deplete_fade >= 1.0 {
                self.deplete_fade = 1.0;
            }
        }

        // Smooth energy growth towards target
        if !self.is_spawning && !self.is_depleting {
            let energy_diff = self.target_energy - self.energy;
            if energy_diff.abs() > 0.1 {
                let growth_direction = if energy_diff > 0.0 { 1.0 } else { -1.0 };
                let growth_amount = self.growth_rate * delta_time * growth_direction;

                if energy_diff.abs() < growth_amount.abs() {
                    self.energy = self.target_energy;
                } else {
                    self.energy += growth_amount;
                }
            }

            // Natural growth towards max energy
            if self.energy < self.max_energy {
                self.energy += self.growth_rate * delta_time * 0.05;
                if self.energy > self.max_energy {
                    self.energy = self.max_energy;
                }
            }

            if self.energy >= self.max_energy {
                self.target_energy = self.max_energy;
            }
        }

        // Size changes based on energy
        let target_size = 3.0 + (self.energy / self.max_energy) * 5.0;
        let size_diff = target_size - self.size;
        if size_diff.abs() > 0.1 {
            self.size += size_diff * delta_time * 2.0;
        }

        // Regeneration when depleted
        if self.energy < 10.0 && !self.is_depleting {
            self.energy += self.regeneration_rate * delta_time * 0.2;
        }
    }

    pub fn is_available(&self) -> bool {
        self.energy > 5.0 && !self.is_depleting && self.spawn_fade > 0.5
    }
}
