use crate::{CONDITION_CHANCE, CONDITION_LIFETIME};
use ::rand::{rngs::StdRng, Rng};
use rand::prelude::IteratorRandom;
use std::time::{Duration, Instant};

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub enum Condition {
    Drought,
    Rainy,
}

impl Condition {
    pub const ALL: [Self; 2] = [Condition::Drought, Condition::Rainy];
}

#[inline(always)]
pub fn update_condition(
    condition: &mut Option<(Condition, (Instant, Duration))>,
    rng: &mut StdRng,
) {
    match condition {
        Some((_, (timestamp, lifetime))) => {
            if &timestamp.elapsed() > lifetime {
                *condition = None;
            }
        }
        None => {
            if rng.gen_range(0.0..1.0) <= unsafe { CONDITION_CHANCE }
            {
                *condition = Some((
                    *Condition::ALL.iter().choose(rng).unwrap(),
                    (
                        Instant::now(),
                        Duration::from_secs(rng.gen_range(unsafe {
                            CONDITION_LIFETIME.clone()
                        })),
                    ),
                ));
            }
        }
    }
}
