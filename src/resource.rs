use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource {
    pub x: f64,
    pub y: f64,
    pub energy: f64,
    pub max_energy: f64,
    pub size: f64,
    pub growth_rate: f64,
    pub regeneration_rate: f64,
    pub age: f64,
}

impl Resource {
    pub fn new(x: f64, y: f64) -> Self {
        let mut rng = thread_rng();

        Self {
            x,
            y,
            energy: rng.gen_range(20.0..80.0),
            max_energy: rng.gen_range(50.0..150.0),
            size: rng.gen_range(3.0..8.0),
            growth_rate: rng.gen_range(0.5..2.0),
            regeneration_rate: rng.gen_range(0.1..0.5),
            age: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.age += delta_time;

        // Natural growth
        if self.energy < self.max_energy {
            self.energy += self.growth_rate * delta_time;
            if self.energy > self.max_energy {
                self.energy = self.max_energy;
            }
        }

        // Size changes based on energy
        self.size = 3.0 + (self.energy / self.max_energy) * 5.0;

        // Regeneration when depleted
        if self.energy < 10.0 {
            self.energy += self.regeneration_rate * delta_time;
        }
    }

    pub fn consume(&mut self, amount: f64) -> f64 {
        let consumed = amount.min(self.energy);
        self.energy -= consumed;

        // If completely depleted, reduce max energy temporarily
        if self.energy <= 0.0 {
            self.max_energy *= 0.8;
            self.energy = 0.0;
        }

        consumed
    }

    pub fn is_available(&self) -> bool {
        self.energy > 5.0
    }

    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }
}
