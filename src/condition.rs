use crate::USER_CONSTANTS;
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
                let user_constants = USER_CONSTANTS.read().unwrap();

                if user_constants.condition_chance > 0.0
                    && (user_constants.condition_chance as usize == 1
                        || rng.gen_range(0.0..1.0)
                            <= user_constants.condition_chance)
                {
                    *condition = Some((
                        *Self::ALL.iter().choose(rng).unwrap(),
                        (
                            Instant::now(),
                            Duration::from_secs(
                                rng.gen_range(
                                    user_constants
                                        .condition_lifetime
                                        .clone(),
                                ),
                            ),
                        ),
                    ));
                }
            }
        }
    }
}
