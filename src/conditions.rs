use crate::{FEWER_PLANTS_CHANCE, MORE_PLANTS_CHANCE};
use ::rand::{rngs::StdRng, Rng};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum Condition {
    FewerPlants,
    MorePlants,
}

impl Condition {
    pub const ALL: [Self; 2] =
        [Condition::FewerPlants, Condition::MorePlants];

    #[inline(always)]
    fn get_chance(&self) -> f32 {
        unsafe {
            match self {
                Condition::FewerPlants => FEWER_PLANTS_CHANCE,
                Condition::MorePlants => MORE_PLANTS_CHANCE,
            }
        }
    }
}

#[inline(always)]
pub fn update_conditions(
    conditions: &mut HashMap<Condition, (Instant, Duration)>,
    rng: &mut StdRng,
) {
    // Keeping those that haven't elapsed yet
    conditions.retain(|_, (instant, duration)| {
        &instant.elapsed() <= duration
    });

    for condition in Condition::ALL {
        if rng.gen_range(0.0..1.0) <= condition.get_chance() {
            conditions.insert(
                condition,
                (
                    Instant::now(),
                    Duration::from_secs(rng.gen_range(20..30)),
                ),
            );
        }
    }
}
