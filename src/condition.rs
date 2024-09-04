use crate::{CONDITION_CHANCE, CONDITION_LIFETIME};
use rand::{prelude::IteratorRandom, rngs::StdRng, Rng};
use std::time::{Duration, Instant};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum Condition {
    Drought,
    Rain,
}

impl Condition {
    pub const ALL: [Self; 2] = [Self::Drought, Self::Rain];

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
                if unsafe { CONDITION_CHANCE } > 0.0
                    && (unsafe { CONDITION_CHANCE } == 1.0
                        || rng.gen_range(0.0..1.0)
                            <= unsafe { CONDITION_CHANCE })
                {
                    *condition = Some((
                        *Condition::ALL.iter().choose(rng).unwrap(),
                        (
                            Instant::now(),
                            Duration::from_secs(rng.gen_range(
                                unsafe { CONDITION_LIFETIME.clone() },
                            )),
                        ),
                    ));
                }
            }
        }
    }
}
