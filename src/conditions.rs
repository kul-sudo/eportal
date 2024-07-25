use crate::Condition;
use ::rand::{rngs::StdRng, Rng};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[inline(always)]
pub fn update_conditions(
    conditions: &mut HashMap<Condition, (Instant, Duration)>,
    rng: &mut StdRng,
) {
    conditions.retain(|_, (instant, duration)| {
        &instant.elapsed() < duration
    });

    if rng.gen_range(0.0..1.0) <= 0.3 {
        conditions.insert(
            Condition::FewerPlants,
            (
                Instant::now(),
                Duration::from_secs(rng.gen_range(20..60)),
            ),
        );
    }
}
