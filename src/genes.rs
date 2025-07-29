use rand::prelude::*;
use rand_distr::Normal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genes {
    pub speed: f64,                  // Movement speed multiplier
    pub sense_range: f64,            // How far the agent can sense resources and other agents
    pub size: f64,                   // Physical size (affects energy consumption and reproduction)
    pub energy_efficiency: f64,      // How efficiently the agent uses energy
    pub reproduction_threshold: f64, // Energy needed to reproduce
    pub mutation_rate: f64,          // How likely genes are to mutate
    pub aggression: f64,             // How likely to attack other agents
    pub color_hue: f64,              // Visual trait for identification
}

impl Genes {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        Self {
            speed: rng.gen_range(0.8..1.5), // Reduced from 0.5..2.0
            sense_range: rng.gen_range(30.0..80.0), // Reduced from 20.0..100.0
            size: rng.gen_range(0.9..1.3), // Reduced from 0.8..1.5
            energy_efficiency: rng.gen_range(0.8..1.2), // Reduced from 0.7..1.3
            reproduction_threshold: rng.gen_range(60.0..120.0), // Reduced from 50.0..150.0
            mutation_rate: rng.gen_range(0.02..0.08), // Reduced from 0.01..0.1
            aggression: rng.gen_range(0.2..0.8), // Reduced from 0.0..1.0
            color_hue: rng.gen_range(0.0..360.0),
        }
    }

    pub fn inherit_from(&self, other: &Genes, mutation_rate: f64) -> Self {
        let mut rng = thread_rng();

        Self {
            speed: self.mutate_gene(self.speed, other.speed, mutation_rate, &mut rng),
            sense_range: self.mutate_gene(
                self.sense_range,
                other.sense_range,
                mutation_rate,
                &mut rng,
            ),
            size: self.mutate_gene(self.size, other.size, mutation_rate, &mut rng),
            energy_efficiency: self.mutate_gene(
                self.energy_efficiency,
                other.energy_efficiency,
                mutation_rate,
                &mut rng,
            ),
            reproduction_threshold: self.mutate_gene(
                self.reproduction_threshold,
                other.reproduction_threshold,
                mutation_rate,
                &mut rng,
            ),
            mutation_rate: self.mutate_gene(
                self.mutation_rate,
                other.mutation_rate,
                mutation_rate,
                &mut rng,
            ),
            aggression: self.mutate_gene(
                self.aggression,
                other.aggression,
                mutation_rate,
                &mut rng,
            ),
            color_hue: self.mutate_gene(self.color_hue, other.color_hue, mutation_rate, &mut rng),
        }
    }

    fn mutate_gene(&self, gene1: f64, gene2: f64, mutation_rate: f64, rng: &mut ThreadRng) -> f64 {
        // Blend genes from both parents
        let blend_factor = rng.gen_range(0.3..0.7);
        let mut gene = gene1 * blend_factor + gene2 * (1.0 - blend_factor);

        // Apply mutation
        if rng.gen::<f64>() < mutation_rate {
            let mutation_dist = Normal::new(0.0, 0.05).unwrap(); // Reduced from 0.1
            let mutation = mutation_dist.sample(rng);
            gene += mutation;
        }

        // Clamp values to reasonable ranges
        match gene {
            speed if speed < 0.1 => 0.1,
            speed if speed > 3.0 => 3.0, // Reduced from 5.0
            sense_range if sense_range < 5.0 => 5.0,
            sense_range if sense_range > 150.0 => 150.0, // Reduced from 200.0
            size if size < 0.3 => 0.3,
            size if size > 2.5 => 2.5, // Reduced from 3.0
            energy_efficiency if energy_efficiency < 0.1 => 0.1,
            energy_efficiency if energy_efficiency > 2.5 => 2.5, // Reduced from 3.0
            reproduction_threshold if reproduction_threshold < 10.0 => 10.0,
            reproduction_threshold if reproduction_threshold > 200.0 => 200.0, // Reduced from 300.0
            mutation_rate if mutation_rate < 0.001 => 0.001,
            mutation_rate if mutation_rate > 0.3 => 0.3, // Reduced from 0.5
            aggression if aggression < 0.0 => 0.0,
            aggression if aggression > 1.0 => 1.0,
            color_hue if color_hue < 0.0 => 0.0,
            color_hue if color_hue > 360.0 => 360.0,
            _ => gene,
        }
    }

    pub fn get_fitness_score(&self) -> f64 {
        // Calculate overall fitness based on gene combinations
        let speed_score = self.speed * 0.3;
        let sense_score = (self.sense_range / 100.0) * 0.2;
        let efficiency_score = self.energy_efficiency * 0.25;
        let size_score = (1.0 / self.size) * 0.15; // Smaller is better for energy efficiency
        let reproduction_score = (1.0 / self.reproduction_threshold) * 50.0 * 0.1;

        speed_score + sense_score + efficiency_score + size_score + reproduction_score
    }
}
