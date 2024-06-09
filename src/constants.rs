use std::f32::consts::SQRT_3;

// TOML
pub const CONFIG_FILE_NAME: &str = "config.toml";

// Base
pub const CELL_ROWS: usize = 89; // May be needed to be changed when the perfect values for the
                                 // evolution process have been determined
pub const BODIES_N: usize = 800;
pub const PLANTS_N: usize = 20000;
pub const PASSIVE_CHANCE: f32 = 0.3;
pub const OBJECT_RADIUS: f32 = 10.0;
/// Used for `get_with_deviation`.
pub const DEVIATION: f32 = 0.1;
pub const COLOR_MIN: u8 = 50;
pub const COLOR_MAX: u8 = 250;
pub const MIN_ENERGY: f32 = 1000.0;
pub const LIFESPAN: f32 = 240.0;

// Average spawn attributes
pub static mut AVERAGE_ENERGY: f32 = 1500.0;
pub static mut AVERAGE_DIVISION_THRESHOLD: f32 = 2300.0;

pub static mut AVERAGE_VISION_DISTANCE: f32 = 0.0; // 100.0
pub const AVERAGE_SPEED: f32 = 1.5;

// Evolution process
pub const PLANT_ENERGY: f32 = 100.0;
pub const MIN_GAP: f32 = 3.0;
pub const COLOR_GAP: f32 = 0.63; // Depends on COLOR_MIN and COLOR_MAX
pub const PLANTS_N_FOR_ONE_STEP: usize = 8; // Mid-game
pub const PLANT_SPAWN_TIME_LIMIT: u64 = 5; // In millis
pub const MIN_TO_REMOVE: usize = 500; // Bodies and plants are removed only it's needed to remove
                                      // more of them than this constant. That's because when the amount of object to remove, the time it
                                      // takes to delete them barely depends on their amount
pub const NEW_SKILL_CHANCE: f32 = 0.07;

// Spending energy
pub const ENERGY_SPENT_CONST_FOR_MASS: f32 = 0.0001;
pub const ENERGY_SPENT_CONST_FOR_SKILLS: f32 = 0.0008;
pub const ENERGY_SPENT_CONST_FOR_VISION_DISTANCE: f32 = 0.00001;
pub const ENERGY_SPENT_CONST_FOR_MOVEMENT: f32 = 0.0002;
pub const CONST_FOR_LIFESPAN: f32 = 0.000005;

// SpeedVirus
pub static mut SPEEDVIRUS_FIRST_GENERATION_INFECTION_CHANCE: f32 = 0.12;
pub static mut SPEEDVIRUS_SPEED_DECREASE: f32 = 0.7;
pub static mut SPEEDVIRUS_ENERGY_SPENT_FOR_HEALING: f32 = 0.08;
pub static mut SPEEDVIRUS_HEAL_ENERGY: f32 = 280.0;

// VisionVirus
pub static mut VISIONVIRUS_FIRST_GENERATION_INFECTION_CHANCE: f32 = 0.1;
pub static mut VISIONVIRUS_VISION_DISTANCE_DECREASE: f32 = 0.7;
pub static mut VISIONVIRUS_ENERGY_SPENT_FOR_HEALING: f32 = 0.08;
pub static mut VISIONVIRUS_HEAL_ENERGY: f32 = 280.5;

// Death
pub const CROSS_LIFESPAN: u64 = 35; // In seconds
pub const PART_OF_PLANTS_TO_REMOVE: f32 = 0.0004;

// Zoom
pub const MAX_ZOOM: f32 = OBJECT_RADIUS;
pub const MIN_ZOOM: f32 = 1.0;

// UI
pub const BODY_INFO_FONT_SIZE: u16 = 17;

// Math
pub const COSINE_OF_30_DEGREES: f32 = SQRT_3 / 2.0;

// Misc
pub const FPS: u64 = 144;
