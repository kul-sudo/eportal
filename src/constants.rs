use std::f32::consts::SQRT_3;

// Base
pub const BODY_EATERS_N: usize = 100;
pub const PLANTS_EATERS_N: usize = 250;
pub const BODIES_N: usize = BODY_EATERS_N + PLANTS_EATERS_N;
pub const PLANTS_N: usize = 20000;
pub const OBJECT_RADIUS: f32 = 10.0;
/// Used for `get_with_deviation`.
pub const DEVIATION: f32 = 0.2;
pub const COLOR_MIN: u8 = 50;
pub const COLOR_MAX: u8 = 250;
pub const MIN_ENERGY: f32 = 1000.0;
pub const LIFESPAN: f32 = 240.0;

// Average spawn attributes
pub const PLANT_EATER_AVERAGE_ENERGY: f32 = 1500.0;
pub const BODY_EATER_AVERAGE_ENERGY: f32 = 3000.0;
pub const PLANT_EATER_AVERAGE_DIVISION_THRESHOLD: f32 = 3000.0;
pub const BODY_EATER_AVERAGE_DIVISION_THRESHOLD: f32 = 3950.0;

pub const AVERAGE_VISION_DISTANCE: f32 = 100.0;
pub const AVERAGE_SPEED: f32 = 1.5;

// Evolution process
pub const PLANT_HP: f32 = 100.0;
pub const MIN_GAP: f32 = 3.0;
pub const COLOR_GAP: f32 = 0.65; // Depends on COLOR_MIN and COLOR_MAX
pub const PLANTS_N_FOR_ONE_STEP: usize = 5; // Mid-game
pub const PLANT_SPAWN_TIME_LIMIT: u64 = 5; // In millis
pub const MIN_TO_REMOVE: usize = 500; // Bodies and plants are removed only it's needed to remove
                                      // more of them than this constant. That's because when the amount of object to remove, the time it
                                      // takes to delete them barely depends on their amount
pub const IQ_INCREASE_CHANCE: f32 = 0.3;
pub const MAX_IQ: u8 = 1;

// Spending energy
pub const ENERGY_SPENT_CONST_FOR_MASS: f32 = 0.00008;
pub const ENERGY_SPENT_CONST_FOR_IQ: f32 = 0.006;
pub const ENERGY_SPENT_CONST_FOR_MOVEMENT: f32 = 0.00008;
pub const CONST_FOR_LIFESPAN: f32 = 0.00001;

// Birth
pub const VISION_DISTANCE_BIRTH_ENERGY_SPENT: f32 = 0.1;
pub const SPEED_BIRTH_ENERGY_SPENT: f32 = 10.0;

// Death
pub const CROSS_LIFESPAN: u64 = 30; // In seconds
pub const PART_OF_PLANTS_TO_REMOVE: f32 = 0.008;

// Zoom
pub const MAX_ZOOM: f32 = OBJECT_RADIUS;
pub const MIN_ZOOM: f32 = 1.0;

// UI
pub const BODY_INFO_FONT_SIZE: u16 = 17;

// Math
pub const COSINE_OF_30_DEGREES: f32 = SQRT_3 / 2.0;

// Misc
pub const FPS: u64 = 144;
