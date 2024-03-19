use std::f32::consts::SQRT_3;

// Base
pub static BODY_EATERS_N: usize = 1000;
pub static PLANTS_EATERS_N: usize = 500;
pub static BODIES_N: usize = BODY_EATERS_N + PLANTS_EATERS_N;
pub static PLANTS_N: usize = 3500;
pub static OBJECT_RADIUS: f32 = 10.0;
/// Used for `get_with_deviation`.
pub static DEVIATION: f32 = 0.15;
pub static COLOR_MIN: u8 = 50;
pub static COLOR_MAX: u8 = 250;

// Average spawn attributes
// pub static AVERAGE_ENERGY: f32 = 90.0;
// pub static AVERAGE_VISION_DISTANCE: f32 = 100.0;
// pub static AVERAGE_SPEED: f32 = 20.0;
// pub static AVERAGE_DIVISION_THRESHOLD: f32 = 150.0;
pub static AVERAGE_ENERGY: f32 = 50.0;
pub static AVERAGE_VISION_DISTANCE: f32 = 300.0;
pub static AVERAGE_SPEED: f32 = 3.0;
pub static AVERAGE_DIVISION_THRESHOLD: f32 = 1000.0;

// Evolution process
pub static PLANT_HP: f32 = 100.0;
pub static MIN_GAP: f32 = 3.0;
pub static COLOR_GAP: f32 = 0.65; // Depends on COLOR_MIN and COLOR_MAX
pub static PLANT_SPAWN_CHANCE: f32 = 1.0; // Mid-game
pub static PLANT_SPAWN_TIME_LIMIT: u64 = 5; // In millis

// Spending energy
pub static ENERGY_SPEND_CONST_FOR_MASS: f32 = 0.0005;
pub static ENERGY_SPEND_CONST_FOR_IQ: f32 = 0.0005;
pub static ENERGY_SPEND_CONST_FOR_VISION: f32 = 0.0005;
pub static ENERGY_SPEND_CONST_FOR_MOVEMENT: f32 = 0.0005;

// Death
pub static CROSS_LIFESPAN: u64 = 5; // In seconds

// Zoom
pub static MAX_ZOOM: f32 = OBJECT_RADIUS;
pub static MIN_ZOOM: f32 = 1.0;

// UI
pub static BODY_INFO_FONT_SIZE: u16 = 17;

// Math
pub static COSINE_OF_30_DEGREES: f32 = SQRT_3 / 2.0;

// Misc
pub static FPS: u64 = 144;
