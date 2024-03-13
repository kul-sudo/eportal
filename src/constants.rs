use std::f32::consts::SQRT_3;

// Base
pub static BODY_EATERS_N: usize = 150;
pub static PLANTS_EATERS_N: usize = 300;
pub static BODIES_N: usize = BODY_EATERS_N + PLANTS_EATERS_N;
pub static PLANTS_N: usize = 2000;
pub static OBJECT_RADIUS: f32 = 10.0;
pub static DEVIATION: f32 = 0.15;

// Average spawn attributes
pub static AVERAGE_ENERGY: f32 = 90.0;
pub static AVERAGE_VISION_DISTANCE: f32 = 100.0;
pub static AVERAGE_SPEED: f32 = 20.0;
pub static AVERAGE_DIVISION_THRESHOLD: f32 = 150.0;

// Evolution process
pub static PLANT_HP: f32 = 15.0;
pub static MIN_GAP: f32 = 3.0;
pub static COLOR_GAP: f32 = 0.685; // Depends on the color limit
pub static PLANT_SPAWN_CHANCE: f32 = 1.0; // Mid-game
pub static PLANT_SPAWN_TIME_LIMIT: u64 = 1; // In millis

// Spending energy
pub static ALIVE_MIN_ENERGY: f32 = 1.0;
pub static ENERGY_SPEND_CONST_FOR_MASS: f32 = 0.01;
pub static ENERGY_SPEND_CONST_FOR_IQ: f32 = 0.01;
pub static CROSS_LIFESPAN: u64 = 5; // In seconds

// Zoom
pub static MAX_ZOOM: f32 = OBJECT_RADIUS;
pub static MIN_ZOOM: f32 = 1.0;
pub static MOUSE_WHEEL_ZOOM_DIVISION: f32 = 10.0;
pub static BODY_JUMP_DELAY: u64 = 150; // In millis

// UI
pub static BODY_INFO_FONT_SIZE: u16 = 17;

// Math
pub static COSINE_OF_30_DEGREES: f32 = SQRT_3 / 2.0;

// Misc
pub static FPS: u64 = 144;
